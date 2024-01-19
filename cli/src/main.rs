use clap::{Parser, Subcommand};

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Some(file) = cli.file {
        let ext = file.split('.').last().unwrap();
        let content: Vec<u8> = tokio::fs::read(&file).await?;

        let part = reqwest::multipart::Part::bytes(content).file_name(format!("file.{}", ext));
        let file = reqwest::multipart::Form::new().part("field_name", part);

        let response = reqwest::Client::new()
            .post("https://devdump.fly.dev/")
            .multipart(file)
            .send()
            .await?;

        println!("{:#?}", response.text().await?);
    }

    match &cli.command {
        Some(Commands::Auth { email }) => {
            println!("Authenticating with email: {:?}", email);
        }
        None => {}
    }

    Ok(())
}
