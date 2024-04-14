use redis::AsyncCommands;
use serde::Deserialize;
use std::{env, sync::Arc, time::Duration};

use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, Request, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Form, Router,
};

use server::email_service::send_email;
use server::shutdown::shutdown_signal;
use server::util::generate_id;

use sqlx::postgres::{self, PgPoolOptions};
use tokio::{fs::File, io::AsyncWriteExt, net::TcpListener};
use tower_http::{
    limit::RequestBodyLimitLayer, services::ServeDir, timeout::TimeoutLayer, trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Clone)]
struct Config {
    hostname: String,
    pgpool: postgres::PgPool,
    redis: Arc<redis::Client>,
}

#[derive(Debug, Deserialize)]
struct LoginForm {
    email: String,
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

    // let pool = PgPoolOptions::new()
    //     .max_connections(10)
    //     .connect_lazy("postgres://postgres:password@localhost:5432/devdump")
    //     .unwrap();
    //
    // // running slqx migration
    // sqlx::migrate!().run(&pool).await.unwrap();
    //
    // // redis
    //
    // let redis_client = Arc::new(redis::Client::open("redis://127.0.0.1/").unwrap());
    //
    tokio::fs::create_dir_all("./upload").await.unwrap();
    //
    // let hostname = env::var("HOST").unwrap_or("http://localhost:3000".to_string());
    // let config = Config {
    //     hostname,
    //     pgpool: pool,
    //     redis: redis_client,
    // };
    // tracing::info!("{:?}", config);

    let app = Router::new()
        .route("/", post(upload))
        .route("/file/:file_name", post(stream))
        .route("/health_check", get(health_check))
        //     .route("/signup", post(signup))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(250 * 1024 * 1024)) // 250MB
        .nest_service("/d", ServeDir::new("upload"))
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::new(Duration::from_secs(5)),
        ));
    // .with_state(config);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tracing::info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

// async fn signup(
//     State(config): State<Config>,
//     Form(login_form): Form<LoginForm>,
// ) -> impl IntoResponse {
//     tracing::info!("signup: {:?}", login_form);
//
//     match sqlx::query!(
//     "INSERT INTO users (id, email, created_on, last_login, status) VALUES ($1, $2, $3, NULL, 'pending_confirmaton')",
//     uuid::Uuid::new_v4(),
//     login_form.email,
//     chrono::Utc::now()
//     ).execute(&config.pgpool).await {
//         Ok(_) => {
//             let token = uuid::Uuid::new_v4().to_string();
//             let mut conn = config.redis.get_async_connection().await.unwrap();
//             conn.set_ex::<String, String, ()>(token.to_owned(), login_form.email.to_owned(), 60 * 30).await.unwrap();
//             let body = format!(
//                 "Click this link to confirm your email: {}/confirm?token={}",
//                 config.hostname, token
//             );
//             send_email(login_form.email, "Confirm your email".to_string(), body).await;
//             return (StatusCode::OK, "ok");
//         }
//         Err(e) => {
//             tracing::error!("error: {}", e);
//             return (StatusCode::INTERNAL_SERVER_ERROR, "error");
//         }
//     };
// }

// State(config): State<Config>,
async fn upload(mut file: Multipart) -> String {
    let mut d = String::new();
    while let Some(f) = file.next_field().await.unwrap() {
        let id = generate_id(6);
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
        file.write_all(&data).await.unwrap();

        tracing::info!("upload file: {}", file_name);
        d = format!("{}/d/{}", "localhost:3000", file_name); // get from env
    }
    return d;
}

// stream to file
async fn stream(
    Path(file_name): Path<String>,
    request: Request,
) -> Result<(), (StatusCode, String)> {
    return server::uploader::stream_to_file(&file_name, request.into_body().into_data_stream())
        .await;
}
