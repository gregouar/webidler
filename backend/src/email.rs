use std::env;

use anyhow::Result;

use lettre::{
    Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::Mailbox,
    transport::smtp::authentication::Credentials,
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
        let sender = Mailbox::new(
            Some("Grind to Rust".into()),
            Address::new("grindtorust", "gregoirenaisse.be")?,
        );

        let email = Message::builder()
            .from(sender.clone())
            .reply_to(sender)
            .to(Mailbox::new(None, target.into_inner().try_into()?))
            .subject(subject)
            .multipart(lettre::message::MultiPart::alternative_plain_html(
                txt_content,
                html_content,
            ))?;

        self.mailer.send(email).await?;

        Ok(())
    }
}
