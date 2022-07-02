use super::auth::Auth;
use super::rand_string;

use crate::prelude::*;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use mongodb::bson::{oid::ObjectId};

impl User {
    /// This method allows to reset the password of a user.
    /// In order for the new password to be saved, it must be passed to a [`Users`] instance.
    /// This function is meant for cases where the user lost their password.
    /// In case the user is authenticated,
    /// you can change it more easily with [`change_password`](`super::auth::Auth::change_password`).
    /// This function will fail in case the password is not secure enough.
    /// ```rust
    /// # use rocket::{State, get};
    /// # use rocket_auth_nosql::{Error, Users};
    /// #[get("/reset-password/<id>/<new_password>")]
    /// async fn reset_password(id: i32, new_password: String, users: &State<Users>) -> Result<(), Error> {
    ///     let mut user = users.get_by_id(id).await?;
    ///     user.set_password(&new_password);
    ///     users.modify(&user).await?;
    ///     Ok(())
    /// }
    /// ```

    pub fn set_password(&mut self, new: &str) -> Result<()> {
        crate::forms::is_secure(new)?;
        let password = new.as_bytes();
        let salt = rand_string(10);
        let config = argon2::Config::default();
        let hash = argon2::hash_encoded(password, salt.as_bytes(), &config).unwrap();
        self.password = hash;
        Ok(())
    }
    /// This method sets the account flag to indicate the email address is verified.
    /// ```rust
    /// # use rocket::{State, get};
    /// # use rocket_auth_nosql::{Error, Users};
    /// #[get("/reset-password/<id>/<new_password>")]
    /// async fn reset_password(id: i32, new_password: String, users: &State<Users>) -> Result<(), Error> {
    ///     let mut user = users.get_by_id(id).await?;
    ///     user.set_password(&new_password);
    ///     users.modify(&user).await?;
    ///     Ok(())
    /// }
    /// ```

    pub fn set_verified(&mut self, token: &str) -> Result<()> {
        if self.verification_token.eq(token) {
            self.is_verified = true;
        } else {
            return Err(Error::VerificationTokenMismatch);
        }
        Ok(())
    }
    /// Activates the account of a user using the token sent via email
    /// ```
    /// # use rocket_auth_nosql::Auth;
    /// # use rocket::post;
    /// # #[post("/activate")]
    /// # fn example(auth: Auth<'_>) {
    ///     auth.change_password("new password");
    /// # }
    /// ```
    // pub fn activate_account(&self, token: &str) {
    //     if self.is_auth() {
    //         let session = self.get_session()?;
    //         let mut user = self.users.get_by_id(session.id).await?;
    //         //user.set_password(password)?;
    //         //self.users.modify(&user).await?;
    //     } else {
    //         Err(Error::UnauthorizedError)
    //     }
    // }

    /// This is an accessor function for the private `id` field.
    /// This field is private so it is not modified by accident when updating a user.
    /// ```rust
    /// # use rocket::{State, get};
    /// # use rocket_auth_nosql::{Error, User};
    /// #[get("/show-my-id")]
    /// fn show_my_id(user: User) -> String {
    ///     format!("Your user_id is: {}", user.id())
    /// }
    /// ```
    pub fn id(&self) -> ObjectId {
        self.id.unwrap()
    }
    /// This is an accessor field for the private `email` field.
    /// This field is private so an email cannot be updated without checking whether it is valid.
    /// ```rust
    /// # use rocket::{State, get};
    /// # use rocket_auth_nosql::{Error, User};
    /// #[get("/show-my-email")]
    /// fn show_my_email(user: User) -> String {
    ///     format!("Your user_id is: {}", user.email())
    /// }
    /// ```
    pub fn email(&self) -> &str {
        &self.email
    }

    /// This functions allows to easily modify the email of a user.
    /// In case the input is not a valid email, it will return an error.
    /// In case the user corresponds to the authenticated client, it's easier to use [`Auth::change_email`].
    /// ```rust
    /// # use rocket::{State, get};
    /// # use rocket_auth_nosql::{Error, Auth};
    /// #[get("/set-email/<email>")]
    /// async fn set_email(email: String, auth: Auth<'_>) -> Result<String, Error> {
    ///     let mut user = auth.get_user().await.unwrap();
    ///     user.set_email(&email)?;
    ///     auth.users.modify(&user).await?;
    ///     Ok("Your user email was changed".into())
    /// }
    /// ```

    pub fn set_email(&mut self, email: &str) -> Result<()> {
        if validator::validate_email(email) {
            Ok(self.email = email.into())
        } else {
            Err(Error::InvalidEmailAddressError)
        }
    }
}

use std::fmt::{self, Debug};

impl Debug for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "User {{ id: {:?}, email: {:?}, is_admin: {:?}, password: \"*****\" }}",
            self.id, self.email, self.is_admin
        )
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = Error;
    async fn from_request(request: &'r Request<'_>) -> Outcome<User, Error> {
        use rocket::outcome::Outcome::*;
        let guard = request.guard().await;
        let auth: Auth = match guard {
            Success(auth) => auth,
            Failure(x) => return Failure(x),
            Forward(x) => return Forward(x),
        };
        if let Some(user) = auth.get_user().await {
            if user.is_verified {
                Outcome::Success(user)
            } else {
                Outcome::Failure((Status::Unauthorized, Error::UnverifiedError))
            }
        } else {
            Outcome::Failure((Status::Unauthorized, Error::UnauthorizedError))
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UnverifiedUser {
    type Error = Error;
    async fn from_request(request: &'r Request<'_>) -> Outcome<UnverifiedUser, Error> {
        use rocket::outcome::Outcome::*;
        let guard = request.guard().await;
        let auth: Auth = match guard {
            Success(auth) => auth,
            Failure(x) => return Failure(x),
            Forward(x) => return Forward(x),
        };
        if let Some(user) = auth.get_user().await {
            Outcome::Success(UnverifiedUser(user))
        } else {
            Outcome::Failure((Status::Unauthorized, Error::UnauthorizedError))
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminUser {
    type Error = Error;
    async fn from_request(request: &'r Request<'_>) -> Outcome<AdminUser, Error> {
        use rocket::outcome::Outcome::*;
        let guard = request.guard().await;
        let auth: Auth = match guard {
            Success(auth) => auth,
            Failure(x) => return Failure(x),
            Forward(x) => return Forward(x),
        };
        if let Some(user) = auth.get_user().await {
            if user.is_admin && user.is_verified {
                return Outcome::Success(AdminUser(user));
            }
        }
        Outcome::Failure((Status::Unauthorized, Error::UnauthorizedError))
    }
}

use std::ops::*;
impl Deref for AdminUser {
    type Target = User;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for AdminUser {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl std::convert::TryFrom<User> for AdminUser {
    type Error = Error;
    fn try_from(value: User) -> Result<Self> {
        if value.is_admin {
            Ok(AdminUser(value))
        } else {
            Err(Error::UnauthorizedError)
        }
    }
}

