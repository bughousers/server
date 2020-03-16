use hyper::body;
use hyper::http::StatusCode;
use hyper::{Body, Method, Request};
use serde_json;

use crate::state::{Channel, State};
use crate::ServerError;

use super::{
    dispatch, ConnectReq, ConnectResp, CreateReq, CreateResp, ReconnectReq, ReconnectResp,
};

const BASE_ADDR: &'static str = "http://localhost";

#[tokio::test]
async fn test_invalid_url() -> Result<(), ServerError> {
    let tx = State::new().serve();
    let req = Request::builder()
        .method(Method::GET)
        .uri(url("invalid"))
        .body(Body::empty())?;
    let resp = dispatch(tx, req).await?;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    Ok(())
}

// Test /connect

#[tokio::test]
async fn test_connect() -> Result<(), ServerError> {
    let tx = State::new().serve();
    let create_resp = valid_create_resp(tx.clone()).await?;
    let req = valid_connect_req(create_resp)?;

    let resp = dispatch(tx, req).await?;
    assert_eq!(resp.status(), StatusCode::OK);
    let resp = body::to_bytes(resp.into_body()).await?;
    serde_json::from_slice::<ConnectResp>(&resp)?;

    Ok(())
}

#[tokio::test]
async fn test_connect_invalid_json() -> Result<(), ServerError> {
    let tx = State::new().serve();
    valid_create_resp(tx.clone()).await?;

    let json = "INVALID";
    let req = Request::builder()
        .method(Method::POST)
        .uri(url("connect"))
        .body(Body::from(json))?;
    let resp = dispatch(tx, req).await?;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    Ok(())
}

#[tokio::test]
async fn test_connect_invalid_user_name() -> Result<(), ServerError> {
    let tx = State::new().serve();
    let create_resp = valid_create_resp(tx.clone()).await?;

    let json = serde_json::to_string(&ConnectReq {
        sessionId: create_resp.sessionId,
        userName: "_INVALID_".into(),
    })?;
    let req = Request::builder()
        .method(Method::POST)
        .uri(url("connect"))
        .body(Body::from(json))?;
    let resp = dispatch(tx, req).await?;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    Ok(())
}

#[tokio::test]
async fn test_connect_invalid_session_id() -> Result<(), ServerError> {
    let tx = State::new().serve();
    valid_create_resp(tx.clone()).await?;

    let json = serde_json::to_string(&ConnectReq {
        sessionId: "_INVALID_".into(),
        userName: "Luigi".into(),
    })?;
    let req = Request::builder()
        .method(Method::POST)
        .uri(url("connect"))
        .body(Body::from(json))?;
    let resp = dispatch(tx, req).await?;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    Ok(())
}

// Test /create

#[tokio::test]
async fn test_create() -> Result<(), ServerError> {
    let tx = State::new().serve();
    let resp = dispatch(tx, valid_create_req()?).await?;
    assert_eq!(resp.status(), StatusCode::OK);
    let resp = body::to_bytes(resp.into_body()).await?;
    serde_json::from_slice::<CreateResp>(&resp)?;
    Ok(())
}

#[tokio::test]
async fn test_create_invalid_json() -> Result<(), ServerError> {
    let tx = State::new().serve();
    let json = "INVALID";
    let req = Request::builder()
        .method(Method::POST)
        .uri(url("create"))
        .body(Body::from(json))?;
    let resp = dispatch(tx, req).await?;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    Ok(())
}

#[tokio::test]
async fn test_create_invalid_user_name() -> Result<(), ServerError> {
    let tx = State::new().serve();
    let json = serde_json::to_string(&CreateReq {
        userName: "_INVALID_".into(),
    })?;
    let req = Request::builder()
        .method(Method::POST)
        .uri(url("create"))
        .body(Body::from(json))?;
    let resp = dispatch(tx, req).await?;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    Ok(())
}

// Test /reconnect

#[tokio::test]
async fn test_reconnect() -> Result<(), ServerError> {
    let tx = State::new().serve();
    let create_resp = valid_create_resp(tx.clone()).await?;
    let req = valid_reconnect_req(create_resp.clone())?;

    let resp = dispatch(tx, req).await?;
    assert_eq!(resp.status(), StatusCode::OK);
    let resp = body::to_bytes(resp.into_body()).await?;
    let resp = serde_json::from_slice::<ReconnectResp>(&resp)?;
    assert_eq!(resp.sessionId, create_resp.sessionId);

    Ok(())
}

#[tokio::test]
async fn test_reconnect_after_connect() -> Result<(), ServerError> {
    let tx = State::new().serve();
    let create_resp = valid_create_resp(tx.clone()).await?;
    let connect_resp = dispatch(tx.clone(), valid_connect_req(create_resp.clone())?).await?;
    let connect_resp = body::to_bytes(connect_resp.into_body()).await?;
    let connect_resp = serde_json::from_slice::<ConnectResp>(&connect_resp)?;

    let json = serde_json::to_string(&ReconnectReq {
        userId: connect_resp.userId,
        authToken: connect_resp.authToken,
    })?;
    let req = Request::builder()
        .method(Method::POST)
        .uri(url("reconnect"))
        .body(Body::from(json))?;
    let resp = dispatch(tx, req).await?;
    assert_eq!(resp.status(), StatusCode::OK);
    let resp = body::to_bytes(resp.into_body()).await?;
    let resp = serde_json::from_slice::<ReconnectResp>(&resp)?;
    assert_eq!(resp.sessionId, create_resp.sessionId);
    assert_eq!(resp.userName, "Luigi");

    Ok(())
}

#[tokio::test]
async fn test_reconnect_invalid_json() -> Result<(), ServerError> {
    let tx = State::new().serve();
    valid_create_resp(tx.clone()).await?;

    let json = "INVALID";
    let req = Request::builder()
        .method(Method::POST)
        .uri(url("reconnect"))
        .body(Body::from(json))?;
    let resp = dispatch(tx, req).await?;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    Ok(())
}

#[tokio::test]
async fn test_reconnect_invalid_user_id() -> Result<(), ServerError> {
    let tx = State::new().serve();
    let create_resp = valid_create_resp(tx.clone()).await?;

    let json = serde_json::to_string(&ReconnectReq {
        userId: "_INVALID_".into(),
        authToken: create_resp.authToken,
    })?;
    let req = Request::builder()
        .method(Method::POST)
        .uri(url("reconnect"))
        .body(Body::from(json))?;
    let resp = dispatch(tx, req).await?;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    Ok(())
}

#[tokio::test]
async fn test_reconnect_invalid_auth_token() -> Result<(), ServerError> {
    let tx = State::new().serve();
    let create_resp = valid_create_resp(tx.clone()).await?;

    let json = serde_json::to_string(&ReconnectReq {
        userId: create_resp.userId,
        authToken: "_INVALID_".into(),
    })?;
    let req = Request::builder()
        .method(Method::POST)
        .uri(url("reconnect"))
        .body(Body::from(json))?;
    let resp = dispatch(tx, req).await?;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    Ok(())
}

// Helper functions

fn url(path: &'static str) -> String {
    format!("{}/{}", BASE_ADDR, path)
}

fn valid_connect_req(create_resp: CreateResp) -> Result<Request<Body>, ServerError> {
    let json = serde_json::to_string(&ConnectReq {
        sessionId: create_resp.sessionId,
        userName: "Luigi".into(),
    })?;
    Ok(Request::builder()
        .method(Method::POST)
        .uri(url("connect"))
        .body(Body::from(json))?)
}

fn valid_create_req() -> Result<Request<Body>, ServerError> {
    let json = serde_json::to_string(&CreateReq {
        userName: "Mario".into(),
    })?;
    Ok(Request::builder()
        .method(Method::POST)
        .uri(url("create"))
        .body(Body::from(json))?)
}

async fn valid_create_resp(tx: Channel) -> Result<CreateResp, ServerError> {
    let resp = dispatch(tx, valid_create_req()?).await?;
    let resp = body::to_bytes(resp.into_body()).await?;
    Ok(serde_json::from_slice::<CreateResp>(&resp)?)
}

fn valid_reconnect_req(create_resp: CreateResp) -> Result<Request<Body>, ServerError> {
    let json = serde_json::to_string(&ReconnectReq {
        userId: create_resp.userId,
        authToken: create_resp.authToken,
    })?;
    Ok(Request::builder()
        .method(Method::POST)
        .uri(url("reconnect"))
        .body(Body::from(json))?)
}
