use std::env;

use anyhow::Result;

use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

use shared::types::Email;

#[derive(Clone)]
pub struct EmailService {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl EmailService {
    pub fn from_env() -> Self {
        let creds = Credentials::new(
            "grindtorust@gregoirenaisse.be".to_owned(),
            env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set"),
        );

        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay("ssl0.ovh.net")
                .unwrap()
                .port(587)
                .credentials(creds)
                .build();

        Self { mailer }
    }

    pub async fn send_email(
        &self,
        target: Email,
        subject: impl Into<String>,
        html_content: String,
        txt_content: String,
    ) -> Result<()> {
        let email = Message::builder()
            .from(
                "Grind to Rust <grindtorust@gregoirenaisse.be>"
                    .parse()
                    .unwrap(),
            )
            .to(Mailbox::new(None, target.into_inner().try_into()?))
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .multipart(lettre::message::MultiPart::alternative_plain_html(
                txt_content,
                html_content,
            ))
            // .body(content)
            .unwrap();

        self.mailer.send(email).await?;

        Ok(())
    }
}
