mod health_check;
mod upload;

use axum::{
    routing::{get, post},
    Router,
};

use tower_http::services::ServeDir;

use health_check::health_check;
use upload::upload;

pub fn router() -> Router {
    return Router::new()
        .route("/upload/:file_name", post(upload))
        .route("/health_check", get(health_check))
        .nest_service("/d", ServeDir::new("upload"));
}
