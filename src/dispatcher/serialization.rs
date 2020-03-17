use hyper::Body;
use serde::{Deserialize, Serialize};

// Request types

#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Req {
    Connect {
        sessionId: String,
        userName: String,
    },
    Create {
        userName: String,
    },
    Authenticated {
        userId: String,
        authToken: String,
        req: AuthenticatedReq,
    },
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum AuthenticatedReq {
    Config { req: ConfigReq },
    Move { req: MoveReq },
    Reconnect,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ConfigReq {
    Participants { participants: Vec<String> },
    Start,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum MoveReq {
    Deploy { piece: String, pos: String },
    Move { change: String },
}

// Response types

#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Resp {
    Connected {
        userId: String,
        authToken: String,
    },
    Created {
        sessionId: String,
        userId: String,
        authToken: String,
    },
    Reconnected {
        sessionId: String,
        userName: String,
    },
}

impl Into<Body> for Resp {
    fn into(self) -> Body {
        Body::from(serde_json::to_string(&self).unwrap())
    }
}
