use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use tracing::warn;
pub const REQUEST_ID_HEADER: &str = "x-request-id";

pub async fn set_request_id(mut req: Request, next: Next) -> Response {
    // if x-request-id is not present in the request headers, generate a new one
    let id = match req.headers().get(REQUEST_ID_HEADER) {
        Some(v) => Some(v.clone()),
        None => {
            let buf = *b"abcdefghijklmnop";
            let request_id = uuid::Uuid::new_v8(buf).to_string();
            match HeaderValue::from_str(&request_id) {
                Ok(v) => {
                    req.headers_mut().insert(REQUEST_ID_HEADER, v.clone());
                    Some(v)
                }
                Err(e) => {
                    warn!("parse request id failed{}", e);
                    None
                }
            }
        }
    };

    let mut response = next.run(req).await;
    let Some(id) = id else {
        return response;
    };
    response.headers_mut().insert(REQUEST_ID_HEADER, id);
    response
}
