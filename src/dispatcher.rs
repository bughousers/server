// Copyright (C) 2020  Kerem Çakırer

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::common::*;
use crate::session::Session;
use crate::sessions::Sessions;
use crate::LISTEN_ADDR;
use hyper::http::response::Builder;
use hyper::{body, header, Body, Method, Response, StatusCode};
use url::Url;

type Request = hyper::Request<Body>;
pub type Result = std::result::Result<Response<Body>, Error>;

#[derive(Debug)]
pub enum Error {
    CannotParse,
    InvalidUserName,
    InvalidSessionId,
    InvalidAuthToken,
    MustBeSessionOwner,
    TooManyUsers,
    TooManyParticipants,
    InvalidParticipantList,
    NotEnoughParticipants,
    GameHasAlreadyStarted,
    GameHasNotStartedYet,
    IllegalMove,
    NotAnActiveParticipant,
    SessionHasEnded,
}

impl Into<Response<Body>> for Error {
    fn into(self) -> Response<Body> {
        bad_request_with_error(match self {
            Error::CannotParse => "Failed to parse request.",
            Error::GameHasAlreadyStarted => "The game has already started.",
            Error::GameHasNotStartedYet => "The game hasn't started yet.",
            Error::IllegalMove => "Illegal move.",
            Error::InvalidAuthToken => "Authentication token is invalid.",
            Error::InvalidParticipantList => "List of participants is invalid.",
            Error::InvalidSessionId => "Session ID is invalid.",
            Error::InvalidUserName => "User name contains illegal characters.",
            Error::MustBeSessionOwner => "This action can only be done by the session owner.",
            Error::NotAnActiveParticipant => "The user is not an active participant.",
            Error::NotEnoughParticipants => "There aren't enough participants yet.",
            Error::SessionHasEnded => "Session has ended.",
            Error::TooManyParticipants => "There are too many participants.",
            Error::TooManyUsers => "There are too many users.",
        })
        .unwrap()
    }
}

pub async fn dispatch(sessions: Sessions, req: Request) -> hyper::Result<Response<Body>> {
    let url = format!("http://{}{}", LISTEN_ADDR, req.uri());
    let url = Url::parse(&url).unwrap();
    let parts: Vec<&str> = url.path_segments().unwrap().collect();
    match parts.split_first() {
        Some((&"v1", rest)) => match dispatch_v1(sessions, rest, req).await {
            Ok(resp) => Ok(resp),
            Err(err) => Ok(err.into()),
        },
        _ => Ok(not_found().unwrap()),
    }
}

async fn dispatch_v1(sessions: Sessions, parts: &[&str], req: Request) -> Result {
    match parts.split_first() {
        Some((&"sessions", rest)) => dispatch_sessions(sessions, rest, req).await,
        _ => not_found(),
    }
}

async fn dispatch_sessions(sessions: Sessions, parts: &[&str], req: Request) -> Result {
    if parts.is_empty() && req.method() == &Method::POST {
        let json = body::to_bytes(req.into_body())
            .await
            .or(Err(Error::CannotParse))?;
        let req = serde_json::from_slice::<req::Create>(&json).or(Err(Error::CannotParse))?;
        let session = Session::new(&req.owner_name);
        if let Some((session, auth_token)) = session {
            let session_id = SessionId::new();
            session.tick();
            sessions.insert(session_id.clone(), session).await;
            to_json(resp::Created {
                session_id,
                user_id: UserId::OWNER,
                auth_token: auth_token,
            })
        } else {
            Err(Error::InvalidUserName)
        }
    } else if let Some((&sid, rest)) = parts.split_first() {
        let session = sessions
            .get(&sid.into())
            .await
            .ok_or(Error::InvalidSessionId)?;
        dispatch_session(session, rest, req).await
    } else {
        not_found()
    }
}

async fn dispatch_session(session: Session, parts: &[&str], req: Request) -> Result {
    if parts.is_empty() {
        if req.method() == &Method::POST {
            let json = body::to_bytes(req.into_body())
                .await
                .or(Err(Error::CannotParse))?;
            let req = serde_json::from_slice::<req::Join>(&json).or(Err(Error::CannotParse))?;
            match req {
                req::Join::Join { user_name } => {
                    let (user_id, auth_token) =
                        session.with(|mut s| s.add_user(&user_name)).await?;
                    to_json(resp::Joined {
                        user_id,
                        user_name,
                        auth_token,
                    })
                }
                req::Join::Rejoin { auth_token } => {
                    let (user_id, user_name) = session
                        .with(|s| {
                            let user_id =
                                s.get_user_id(&auth_token).ok_or(Error::InvalidAuthToken)?;
                            let user_name = s.get_user_name(&user_id).unwrap().clone();
                            Ok((user_id, user_name))
                        })
                        .await?;
                    to_json(resp::Joined {
                        user_id,
                        user_name,
                        auth_token,
                    })
                }
            }
        } else if req.method() == &Method::DELETE {
            not_found() // TODO: Implement
        } else {
            not_found()
        }
    } else {
        match parts.split_first() {
            Some((&"games", rest)) => dispatch_games(session, rest, req).await,
            Some((&"participants", rest)) => dispatch_participants(session, rest, req).await,
            Some((&"sse", rest)) => dispatch_sse(session, rest, req).await,
            _ => not_found(),
        }
    }
}

async fn dispatch_games(session: Session, parts: &[&str], req: Request) -> Result {
    if parts.is_empty() && req.method() == &Method::POST {
        let json = body::to_bytes(req.into_body())
            .await
            .or(Err(Error::CannotParse))?;
        let req = serde_json::from_slice::<req::Start>(&json).or(Err(Error::CannotParse))?;
        session.with(|mut s| s.start(&req.auth_token)).await?;
        no_content()
    } else if let Some((&gid, rest)) = parts.split_first() {
        dispatch_game(session, rest, req, gid).await
    } else {
        not_found()
    }
}

async fn dispatch_participants(session: Session, parts: &[&str], req: Request) -> Result {
    if parts.is_empty() && req.method() == &Method::PUT {
        let json = body::to_bytes(req.into_body())
            .await
            .or(Err(Error::CannotParse))?;
        let req = serde_json::from_slice::<req::Participants>(&json).or(Err(Error::CannotParse))?;
        session
            .with(|mut s| s.set_participants(&req.auth_token, req.participants))
            .await?;
        no_content()
    } else {
        not_found()
    }
}

async fn dispatch_sse(session: Session, parts: &[&str], req: Request) -> Result {
    if parts.is_empty() && req.method() == &Method::GET {
        let rx = session.with(|mut s| s.subscribe()).await;
        Ok(event_stream_builder().body(Body::wrap_stream(rx)).unwrap())
    } else {
        not_found()
    }
}

async fn dispatch_game(session: Session, parts: &[&str], req: Request, game_id: &str) -> Result {
    match parts.split_first() {
        Some((&"board", rest)) => dispatch_board(session, rest, req, game_id).await,
        _ => not_found(),
    }
}

async fn dispatch_board(session: Session, parts: &[&str], req: Request, game_id: &str) -> Result {
    if parts.is_empty() && req.method() == &Method::POST {
        let json = body::to_bytes(req.into_body())
            .await
            .or(Err(Error::CannotParse))?;
        let req = serde_json::from_slice::<req::Board>(&json).or(Err(Error::CannotParse))?;
        match req {
            req::Board::Deploy {
                auth_token,
                piece,
                pos,
            } => {
                session
                    .with(|mut s| s.deploy_piece(&auth_token, piece, pos))
                    .await?;
                no_content()
            }
            req::Board::Move { auth_token, change } => {
                session
                    .with(|mut s| s.move_piece(&auth_token, change))
                    .await?;
                no_content()
            }
        }
    } else {
        not_found()
    }
}

// Helper functions

// TODO: Don't set Access-Control-Allow-Origin to *
fn builder() -> Builder {
    Response::builder().header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
}

fn event_stream_builder() -> Builder {
    builder()
        .header(header::CONNECTION, "keep-alive")
        .header(header::CONTENT_TYPE, "text/event-stream")
}

fn json_builder() -> Builder {
    builder().header(header::CONTENT_TYPE, "application/json; charset=UTF-8")
}

fn to_json<T: Into<Body>>(t: T) -> Result {
    Ok(json_builder().body(t.into()).unwrap())
}

fn no_content() -> Result {
    Ok(builder()
        .status(StatusCode::NO_CONTENT)
        .body(Body::empty())
        .unwrap())
}

fn bad_request() -> Result {
    Ok(builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::empty())
        .unwrap())
}

fn bad_request_with_error(error: &'static str) -> Result {
    Ok(json_builder()
        .status(StatusCode::BAD_REQUEST)
        .body(resp::Error { error }.into())
        .unwrap())
}

fn not_found() -> Result {
    Ok(builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .unwrap())
}
