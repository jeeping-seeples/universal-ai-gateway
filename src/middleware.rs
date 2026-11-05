use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

pub static X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

pub async fn request_id_middleware(mut req: Request, next: Next) -> Response {
    let request_id = match req.headers().get(&X_REQUEST_ID) {
        Some(val) => val.clone(),
        None => {
            let id = Uuid::new_v4().to_string();
            HeaderValue::from_str(&id).unwrap_or_else(|_| HeaderValue::from_static("unknown"))
        }
    };

    req.headers_mut()
        .insert(X_REQUEST_ID.clone(), request_id.clone());

    let method = req.method().clone();
    let uri = req.uri().path().to_string();
    let id_str = request_id.to_str().unwrap_or("unknown").to_string();

    tracing::info!(request_id = %id_str, method = %method, path = %uri, "Handling request");

    let mut res = next.run(req).await;
    res.headers_mut().insert(X_REQUEST_ID.clone(), request_id);
    res
}
