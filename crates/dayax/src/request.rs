use std::collections::HashMap;

use axum::async_trait;
use axum::extract::FromRequest;
use axum::extract::FromRequestParts;
use axum::extract::Path;
use axum::extract::Query;
use axum::BoxError;
use axum::Form;

use axum::http::Request;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
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
    body: serde_json::Value,
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
    type Rejection = Response;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();
        // Infallible
        let method = Method::from_request_parts(&mut parts, state)
            .await
            .unwrap()
            .to_string();
        let Query(search_params) = Query::from_request_parts(&mut parts, state)
            .await
            .map_err(IntoResponse::into_response)?;
        // Infallible
        let uri = Uri::from_request_parts(&mut parts, state)
            .await
            .unwrap()
            .to_string();
        let Path(path) = Path::from_request_parts(&mut parts, state)
            .await
            .map_err(IntoResponse::into_response)?;

        // Infallible
        let headers: HashMap<_, _> = HeaderMap::from_request_parts(&mut parts, state)
            .await
            .unwrap()
            .iter()
            .map(|(key, value)| {
                (
                    key.to_string(),
                    value.to_str().unwrap_or_default().to_string(),
                )
            })
            .collect();
        let req = Request::from_parts(parts, body);
        let mime = headers
            .get(http::header::CONTENT_TYPE.as_str())
            .map(|x| x.parse::<mime::Mime>().ok())
            .flatten();
        let body = match mime {
            Some(x) if mime::APPLICATION_JSON == x => {
                Json::from_request(req, state)
                    .await
                    .map_err(IntoResponse::into_response)?
                    .0
            }
            Some(x) if x == mime::APPLICATION_WWW_FORM_URLENCODED => {
                Form::from_request(req, state)
                    .await
                    .map_err(IntoResponse::into_response)?
                    .0
            }
            _ => String::from_request(req, state)
                .await
                .map(serde_json::Value::String)
                .map_err(IntoResponse::into_response)?,
        };
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
