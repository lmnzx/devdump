/* TODO
* curl --request POST \
*  --url https://devdump.fly.dev/ \
*  --header 'Content-Type: multipart/form-data' \
*  --header 'User-Agent: devdump/cli' \
*  --form =@file.jpeg
*/

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file: String = env::args().nth(1).unwrap();
    let ext = file.split('.').last().unwrap();
    println!("path: {}\next: {}", file, ext);
    let content: Vec<u8> = tokio::fs::read(&file).await?;

    let part = reqwest::multipart::Part::bytes(content).file_name(format!("file.{}", ext));
    let file = reqwest::multipart::Form::new().part("field_name", part);

    let response = reqwest::Client::new()
        .post("http://localhost:3000/")
        .multipart(file)
        .send()
        .await?;

    println!("{:#?}", response);
    println!("{:#?}", response.text().await?);

    Ok(())
}
