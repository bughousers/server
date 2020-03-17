use hyper::http::StatusCode;

use super::super::state::State;
use super::*;

#[tokio::test]
async fn with_invalid_url() -> Result<(), ServerError> {
    let tx = State::new().serve();

    let req = Request::builder()
        .method(Method::GET)
        .uri("http://localhost/invalid")
        .body(Body::empty())?;

    let resp = dispatch(tx, req).await?;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    Ok(())
}

#[tokio::test]
async fn api_with_invalid_json() -> Result<(), ServerError> {
    let tx = State::new().serve();

    let req = Request::builder()
        .method(Method::POST)
        .uri("http://localhost/api")
        .body(Body::from("INVALID"))?;

    let resp = dispatch(tx, req).await?;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    Ok(())
}

#[tokio::test]
async fn api_connect() -> Result<(), ServerError> {
    let tx = State::new().serve();

    let create_req = Req::Create {
        userName: "Mario".into(),
    };
    let create_req = serde_json::to_string(&create_req)?;
    let create_req = Body::from(create_req);

    let create_resp = dispatch_api(tx.clone(), create_req).await?;
    let create_resp = body::to_bytes(create_resp.into_body()).await?;
    let create_resp = serde_json::from_slice::<Resp>(&create_resp)?;

    if let Resp::Created {
        sessionId,
        userId,
        authToken,
    } = create_resp
    {
        let req = Req::Connect {
            sessionId: sessionId,
            userName: "Mario".into(),
        };
        let req = serde_json::to_string(&req)?;
        let req = Body::from(req);

        let resp = dispatch_api(tx, req).await?;
        assert_eq!(resp.status(), StatusCode::OK);

        let resp = body::to_bytes(resp.into_body()).await?;
        let res = match serde_json::from_slice::<Resp>(&resp)? {
            Resp::Connected { userId, authToken } => true,
            _ => false,
        };
        assert!(res);
    }

    Ok(())
}

#[tokio::test]
async fn api_connect_with_invalid_name() -> Result<(), ServerError> {
    let tx = State::new().serve();

    let create_req = Req::Create {
        userName: "Mario".into(),
    };
    let create_req = serde_json::to_string(&create_req)?;
    let create_req = Body::from(create_req);

    let create_resp = dispatch_api(tx.clone(), create_req).await?;
    let create_resp = body::to_bytes(create_resp.into_body()).await?;
    let create_resp = serde_json::from_slice::<Resp>(&create_resp)?;

    if let Resp::Created {
        sessionId,
        userId,
        authToken,
    } = create_resp
    {
        let req = Req::Connect {
            sessionId: sessionId,
            userName: "_INVALID_".into(),
        };
        let req = serde_json::to_string(&req)?;
        let req = Body::from(req);

        let resp = dispatch_api(tx, req).await?;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    Ok(())
}

#[tokio::test]
async fn api_connect_with_invalid_session_id() -> Result<(), ServerError> {
    let tx = State::new().serve();

    let create_req = Req::Create {
        userName: "Mario".into(),
    };
    let create_req = serde_json::to_string(&create_req)?;
    let create_req = Body::from(create_req);

    let create_resp = dispatch_api(tx.clone(), create_req).await?;
    let create_resp = body::to_bytes(create_resp.into_body()).await?;
    let create_resp = serde_json::from_slice::<Resp>(&create_resp)?;

    if let Resp::Created {
        sessionId,
        userId,
        authToken,
    } = create_resp
    {
        let req = Req::Connect {
            sessionId: "_INVALID_".into(),
            userName: "Luigi".into(),
        };
        let req = serde_json::to_string(&req)?;
        let req = Body::from(req);

        let resp = dispatch_api(tx, req).await?;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    Ok(())
}

#[tokio::test]
async fn api_create() -> Result<(), ServerError> {
    let tx = State::new().serve();

    let req = Req::Create {
        userName: "Mario".into(),
    };
    let req = serde_json::to_string(&req)?;
    let req = Body::from(req);

    let resp = dispatch_api(tx, req).await?;
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = body::to_bytes(resp.into_body()).await?;
    let res = match serde_json::from_slice::<Resp>(&resp)? {
        Resp::Created {
            sessionId,
            userId,
            authToken,
        } => true,
        _ => false,
    };
    assert!(res);

    Ok(())
}

#[tokio::test]
async fn api_create_with_invalid_user_name() -> Result<(), ServerError> {
    let tx = State::new().serve();

    let req = Req::Create {
        userName: "_INVALID_".into(),
    };
    let req = serde_json::to_string(&req)?;
    let req = Body::from(req);

    let resp = dispatch_api(tx, req).await?;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    Ok(())
}

#[tokio::test]
async fn api_reconnect_after_connect() -> Result<(), ServerError> {
    let tx = State::new().serve();

    let create_req = Req::Create {
        userName: "Mario".into(),
    };
    let create_req = serde_json::to_string(&create_req)?;
    let create_req = Body::from(create_req);

    let create_resp = dispatch_api(tx.clone(), create_req).await?;
    let create_resp = body::to_bytes(create_resp.into_body()).await?;
    let create_resp = serde_json::from_slice::<Resp>(&create_resp)?;

    if let Resp::Created {
        sessionId,
        userId,
        authToken,
    } = create_resp
    {
        let connect_req = Req::Connect {
            sessionId: sessionId,
            userName: "Mario".into(),
        };
        let connect_req = serde_json::to_string(&connect_req)?;
        let connect_req = Body::from(connect_req);

        let connect_resp = dispatch_api(tx.clone(), connect_req).await?;
        let connect_resp = body::to_bytes(connect_resp.into_body()).await?;
        let connect_resp = serde_json::from_slice::<Resp>(&connect_resp)?;

        if let Resp::Connected { userId, authToken } = connect_resp {
            let req = Req::Authenticated {
                userId: userId,
                authToken: authToken,
                req: AuthenticatedReq::Reconnect,
            };
            let req = serde_json::to_string(&req)?;
            let req = Body::from(req);

            let resp = dispatch_api(tx, req).await?;
            assert_eq!(resp.status(), StatusCode::OK);

            let resp = body::to_bytes(resp.into_body()).await?;
            let res = match serde_json::from_slice::<Resp>(&resp)? {
                Resp::Reconnected {
                    sessionId,
                    userName,
                } => true,
                _ => false,
            };
            assert!(res);
        }
    }

    Ok(())
}

#[tokio::test]
async fn api_reconnect_after_create() -> Result<(), ServerError> {
    let tx = State::new().serve();

    let create_req = Req::Create {
        userName: "Mario".into(),
    };
    let create_req = serde_json::to_string(&create_req)?;
    let create_req = Body::from(create_req);

    let create_resp = dispatch_api(tx.clone(), create_req).await?;
    let create_resp = body::to_bytes(create_resp.into_body()).await?;
    let create_resp = serde_json::from_slice::<Resp>(&create_resp)?;

    if let Resp::Created {
        sessionId,
        userId,
        authToken,
    } = create_resp
    {
        let req = Req::Authenticated {
            userId: userId,
            authToken: authToken,
            req: AuthenticatedReq::Reconnect,
        };
        let req = serde_json::to_string(&req)?;
        let req = Body::from(req);

        let resp = dispatch_api(tx, req).await?;
        assert_eq!(resp.status(), StatusCode::OK);

        let resp = body::to_bytes(resp.into_body()).await?;
        let res = match serde_json::from_slice::<Resp>(&resp)? {
            Resp::Reconnected {
                sessionId,
                userName,
            } => true,
            _ => false,
        };
        assert!(res);
    }

    Ok(())
}
