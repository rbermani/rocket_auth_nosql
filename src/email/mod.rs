use crate::prelude::*;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Transport, SmtpTransport, Message};
use std::format;

const SMTP_SERVER: &str = "smtp.gmail.com";
const SMTP_USERNAME: &str = "testuser";
const SMTP_PASSWORD: &str = "testpass";
const FROM_ADDRESS: &str = "Test User <testuser@devnull.null>";
const NEW_ACCOUNT_ACTIVATION_SUBJ: &str = "You have created a new account that requires activation.";

pub struct Mailer {
    mailer: SmtpTransport
}

impl Mailer {
    fn new() -> Self {
        Default::default()
    }

    fn send_activation_email(&self, to: &str, token: &str) -> Result<()> {
        let email = Message::builder()
            .from(FROM_ADDRESS.parse().unwrap())
            .to(to.parse().unwrap())
            .subject(NEW_ACCOUNT_ACTIVATION_SUBJ)
            .body(format!("To activate your account, use the following token: {}", token))
            .unwrap();
         match self.mailer.send(&email) {
             Ok(_) => Ok(()),
             Err(_) => Err(Error::SmtpRequestError),
         }

    }
}

impl Default for Mailer {
    fn default() -> Self {
        let mailer = SmtpTransport::relay(SMTP_SERVER)
            .unwrap()
            .credentials(Credentials::new(SMTP_USERNAME.into(), SMTP_PASSWORD.into()))
            .build();

        Mailer { mailer }
    }
}