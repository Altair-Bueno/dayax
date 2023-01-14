use std::collections::HashMap;

use axum::async_trait;
use axum::extract::FromRequest;
use axum::extract::FromRequestParts;
use axum::extract::Path;
use axum::extract::Query;
use axum::BoxError;

use axum::http::Request;
use http::HeaderMap;
use http::Method;
use http::Uri;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DayaxRequest {
    method: String,
    #[serde(rename = "searchParams")]
    search_params: HashMap<String, String>,
    uri: String,
    path: HashMap<String, String>,
    headers: HashMap<String, String>,
    body: String,
}

#[async_trait]
impl<S, B> FromRequest<S, B> for DayaxRequest
where
    B: Send + 'static,
    S: Send + Sync,
    B: http_body::Body + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
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

        let headers = HeaderMap::from_request_parts(&mut parts, state)
            .await
            .unwrap()
            .into_iter()
            .map(|(key, value)| {
                (
                    key.map(|x| x.to_string()).unwrap_or_default(),
                    value.to_str().unwrap_or_default().to_owned(),
                )
            })
            .collect();
        // TODO extract everything
        let req = Request::from_parts(parts, body);
        let body = String::from_request(req, state).await.unwrap();
        Ok(DayaxRequest {
            method,
            search_params,
            uri,
            path,
            headers,
            body,
        })
    }
}
