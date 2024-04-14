use server::routes::router;
use server::shutdown::shutdown_signal;

use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tokio::fs::create_dir_all("./upload").await.unwrap();

    let app = router();

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tracing::info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}
