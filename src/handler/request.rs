use std::collections::HashMap;

use axum::async_trait;
use axum::extract::FromRequest;
use axum::extract::FromRequestParts;
use axum::extract::Path;
use axum::extract::Query;
use axum::http;
use axum::http::Request;
use http::Method;
use http::Uri;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HandlerRequest {
    method: String,
    #[serde(rename = "searchParams")]
    search_params: HashMap<String, String>,
    uri: String,
    path: HashMap<String, String>,
}

#[async_trait]
impl<S, B> FromRequest<S, B> for HandlerRequest
where
    B: Send + 'static,
    S: Send + Sync,
{
    type Rejection = String;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();
        let method = Method::from_request_parts(&mut parts, state)
            .await
            .unwrap()
            .to_string();
        let Query(search_params) = Query::from_request_parts(&mut parts, state)
            .await
            .map_err(|x| x.to_string())?;
        let uri = Uri::from_request_parts(&mut parts, state)
            .await
            .unwrap()
            .to_string();
        let Path(path) = Path::from_request_parts(&mut parts, state).await.unwrap();
        // TODO extract everything
        let _req = Request::from_parts(parts, body);
        Ok(HandlerRequest {
            method,
            search_params,
            uri,
            path,
        })
    }
}
