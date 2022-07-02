use std::*;

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// This error occurs when attempting to create a user with an invalid email address.
    #[error("That is not a valid email address.")]
    InvalidEmailAddressError,

    /// Thrown when the requested user does not exists.
    #[error("Could not find any user that fits the specified requirements.")]
    UserNotFoundError,

    /// This error is thrown when trying to retrieve `Users` but it isn't being managed by the app.
    /// It can be fixed adding `.manage(users)` to the app, where `users` is of type `Users`.
    #[error("UnmanagedStateError: failed retrieving `Users`. You may be missing `.manage(users)` in your app.")]
    UnmanagedStateError,

    #[error("UnauthenticatedError: The operation failed because the client is not authenticated.")]
    UnauthenticatedError,
    /// This error occurs when a user tries to log in, but their account doesn't exists.
    #[error("The email \"{0}\" is not registered. Try signing up first.")]
    EmailDoesNotExist(String),
    /// This error is thrown when a user tries to sign up with an email that already exists.
    #[error("That email address already exists. Try logging in.")]
    EmailAlreadyExists,
    /// This error occurs when the user does exists, but their password was incorrect.
    #[error("Incorrect email or password")]
    UnauthorizedError,
    /// This error occurs when a request to verify a client's email address contains an invalid verification token
    #[error("Invalid account verification token")]
    VerificationTokenMismatch,
    /// This error occurs when the user has authenticated but the account is not verified
    #[error("Unverified email address")]
    UnverifiedError,
    /// This error occurs when the SMTP server request encountered an error
    #[error("SMTP Transport Error")]
    SmtpRequestError,
    /// A wrapper around [`validator::ValidationError`].
    #[error("{0}")]
    FormValidationError(#[from] validator::ValidationError),

    /// A wrapper around [`validator::ValidationErrors`].
    #[error("FormValidationErrors: {0}")]
    FormValidationErrors(#[from] validator::ValidationErrors),

    /// A wrapper around [`argon2::Error`].
    #[error("Argon2ParsingError: {0}")]
    Argon2ParsingError(#[from] argon2::Error),

    /// A wrapper around [`redis::RedisError`].
    #[cfg(feature = "redis")]
    #[error("RedisError")]
    RedisError(#[from] redis::RedisError),

    /// A wrapper around [`serde_json::Error`].
    #[error("SerdeError: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("MongoDBError")]
    MongoDBError(#[from] mongodb::error::Error)
}

use self::Error::*;
impl Error {
    fn message(&self) -> String {
        match self {
            MongoDBError(err) => format!("{}", err),
            InvalidEmailAddressError
            | VerificationTokenMismatch
            | EmailAlreadyExists
            | UnauthorizedError
            | SmtpRequestError
            | UserNotFoundError => format!("{}", self),
            FormValidationErrors(source) => {
                source
                    .field_errors()
                    .into_iter()
                    .map(|(_, error)| error)
                    .map(IntoIterator::into_iter)
                    .map(|errs| {
                        errs //
                            .map(|err| &err.code)
                            .fold(String::new(), |a, b| a + b)
                    })
                    .fold(String::new(), |a, b| a + &b)
            }
            #[cfg(debug_assertions)]
            e => return format!("{}", e),
            #[allow(unreachable_patterns)]
            _ => "undefined".into(),
        }
    }
}

use rocket::http::ContentType;
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use serde_json::*;
use std::io::Cursor;

impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let payload = to_string(&json!({
            "status": "error",
            "message": self.message(),
        }))
        .unwrap();
        Response::build()
            .sized_body(payload.len(), Cursor::new(payload))
            .header(ContentType::new("application", "json"))
            .ok()
    }
}
