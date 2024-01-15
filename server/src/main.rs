use axum::{
    extract::{DefaultBodyLimit, Multipart},
    routing::post,
    Router,
};
use tokio::{fs::File, io::AsyncWriteExt, net::TcpListener};
use tower_http::limit::RequestBodyLimitLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use ulid::Ulid;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/", post(upload))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(250 * 1024 * 1024)) // 250MB
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn upload(mut file: Multipart) {
    while let Some(f) = file.next_field().await.unwrap() {
        let id = Ulid::new().to_string();
        let file_name = f.file_name().unwrap().to_string();
        let file_name = file_name.split('.').last();
        let file_name = match file_name {
            Some(file_name) => file_name,
            None => {
                tracing::error!("file name is empty");
                continue;
            }
        };
        let file_name = format!("{}.{}", id, file_name);
        let data = f.bytes().await.unwrap();

        let mut file = File::create(format!("./upload/{}", file_name))
            .await
            .unwrap();
        file.write(&data).await.unwrap();

        tracing::info!("upload file: {}", file_name);
    }
}
