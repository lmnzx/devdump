use std::{env, time::Duration};

use axum::{
    extract::{DefaultBodyLimit, Multipart, State},
    routing::post,
    Router,
};
use tokio::{fs::File, io::AsyncWriteExt, net::TcpListener, signal};
use tower_http::{
    limit::RequestBodyLimitLayer, services::ServeDir, timeout::TimeoutLayer, trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use ulid::Ulid;

#[derive(Debug, Clone)]
struct Config {
    hostname: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tokio::fs::create_dir_all("./upload").await.unwrap();

    let hostname = env::var("HOST").unwrap_or("http://localhost:3000".to_string());
    let config = Config { hostname };
    tracing::info!("{:?}", config);

    let app = Router::new()
        .route("/", post(upload))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(250 * 1024 * 1024)) // 250MB
        .nest_service("/d", ServeDir::new("upload"))
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::new(Duration::from_secs(5)),
        ))
        .with_state(config);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C signal handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM signal handler")
            .recv()
            .await
            .expect("failed to receive SIGTERM signal")
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("shutdown signal received"),
        _ = terminate => tracing::info!("shutdown signal received"),
    }
}

async fn upload(State(config): State<Config>, mut file: Multipart) -> String {
    let mut d = String::new();
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
        d = format!("{}/d/{}", config.hostname, file_name); // get from env
    }
    d
}
