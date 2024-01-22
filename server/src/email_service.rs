use lettre::{
    message::header::ContentType, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

pub async fn send_email(to: String, sub: String, body: String) {
    let email = Message::builder()
        .from("test@devdump.com".parse().unwrap())
        .in_reply_to("test@devdump.com".parse().unwrap())
        .to(to.parse().unwrap())
        .subject(sub)
        .header(ContentType::TEXT_HTML)
        .body(body)
        .unwrap();

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::unencrypted_localhost();

    match mailer.send(email).await {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {:?}", e),
    }
}
