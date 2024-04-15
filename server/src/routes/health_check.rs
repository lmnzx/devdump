use axum::http::StatusCode;

/// a simple health_check service
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}
