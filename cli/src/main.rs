use clap::{Parser, Subcommand};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    file: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// authenticate
    Auth { email: Option<String> },
}

fn file_to_request_body(file: File) -> reqwest::Body {
    let stream = FramedRead::new(file, BytesCodec::new());
    let body = reqwest::Body::wrap_stream(stream);
    return body;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Some(path) = cli.file {
        let file = File::open(&path).await?;
        let file_name = path.split('/').last().ok_or("")?;
        let url = format!("http://localhost:3000/upload/{}", file_name);

        let response = reqwest::Client::new()
            .post(url)
            .body(file_to_request_body(file))
            .send()
            .await?;

        println!("{}", response.status());
    }

    match &cli.command {
        Some(Commands::Auth { email }) => {
            println!("Authenticating with email: {:?}", email);
        }
        None => {}
    }

    Ok(())
}
