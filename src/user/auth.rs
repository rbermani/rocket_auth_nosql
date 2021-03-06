use crate::prelude::*;
use rocket::http::Status;
use rocket::http::{Cookie, CookieJar};
use rocket::request::FromRequest;
use rocket::request::Outcome;
use rocket::Request;
use rocket::State;
use serde_json::json;
use std::time::Duration;

/// The [`Auth`] guard allows the user to log in, log out, sign up, modify, and delete the currently (un)authenticated user.
/// The [`User`] guard only succeeds if the user has verified their account and fully authenticated.
/// /// The [`UnverifiedUser`] guard succeeds if the user is authenticated, but has not yet verified their account.
/// For more information see [`Auth`].
///  A working example:
/// ```rust,no_run
///
/// use rocket::{*, form::Form};
/// use rocket_auth_nosql::{Users, Error, Auth, Signup, Login};
///
/// #[post("/signup", data="<form>")]
/// async fn signup(form: Form<Signup>, auth: Auth<'_>) {
///     auth.signup(&form).await;
///     auth.login(&form.into());
/// }
///
/// #[post("/login", data="<form>")]
/// fn login(form: Form<Login>, auth: Auth) {
///     auth.login(&form);
/// }
///
/// #[get("/logout")]
/// fn logout(auth: Auth) {
///     auth.logout();
/// }
/// #[tokio::main]
/// async fn main() -> Result<(), Error>{
///     let users = Users::open_mongodb("mongodb://localhost:27017", DATABASE).await?;
///
///     rocket::build()
///         .mount("/", routes![signup, login, logout])
///         .manage(users)
///         .launch()
///         .await;
///     Ok(())
/// }
/// ```
#[allow(missing_docs)]
pub struct Auth<'a> {
    /// `Auth` includes in its fields a [`Users`] instance. Therefore, it is not necessary to retrieve `Users` when using this guard.
    pub users: &'a State<Users>,
    pub cookies: &'a CookieJar<'a>,
    pub session: Option<Session>,
}

#[async_trait]
impl<'r> FromRequest<'r> for Auth<'r> {
    type Error = Error;
    async fn from_request(req: &'r Request<'_>) -> Outcome<Auth<'r>, Error> {
        let session: Option<Session> = if let Outcome::Success(users) = req.guard().await {
            Some(users)
        } else {
            None
        };

        let users: &State<Users> = if let Outcome::Success(users) = req.guard().await {
            users
        } else {
            return Outcome::Failure((Status::InternalServerError, Error::UnmanagedStateError));
        };

        Outcome::Success(Auth {
            users,
            session,
            cookies: req.cookies(),
        })
    }
}

impl<'a> Auth<'a> {
    /// Logs in the user through a parsed form or json.
    /// The session is set to expire in one year by default.
    /// For a custom expiration date use [`Auth::login_for`].
    /// ```rust
    /// # use rocket::{get, post, form::Form};
    /// # use rocket_auth_nosql::{Auth, Login};
    /// #[post("/login", data="<form>")]
    /// fn login(form: Form<Login>, auth: Auth) {
    ///     auth.login(&form);
    /// }
    /// ```

    pub async fn login(&self, form: &Login) -> Result<()> {
        let key = self.users.login(form).await?;
        let user = self.users.get_by_email(&form.email).await?;
        let session = Session {
            id: user.id.unwrap(),
            email: user.email,
            auth_key: key,
            time_stamp: now(),
        };
        let to_str = format!("{}", json!(session));
        self.cookies.add_private(Cookie::new("rocket_auth_nosql", to_str));
        Ok(())
    }

    /// Logs a user in for the specified period of time.
    /// ```rust
    /// # use rocket::{post, form::Form};
    /// # use rocket_auth_nosql::{Login, Auth};
    /// # use std::time::Duration;
    /// #[post("/login", data="<form>")]
    /// fn login(form: Form<Login>, auth: Auth) {
    ///     let one_hour = Duration::from_secs(60 * 60);
    ///     auth.login_for(&form, one_hour);
    /// }
    /// ```

    pub async fn login_for(&self, form: &Login, time: Duration) -> Result<()>  {
        let key = self.users.login_for(form, time).await?;
        let user = self.users.get_by_email(&form.email).await?;

        let session = Session {
            id: user.id.unwrap(),
            email: user.email,
            auth_key: key,
            time_stamp: now(),
        };
        let to_str = format!("{}", json!(session));
        let cookie = Cookie::new("rocket_auth_nosql", to_str);
        self.cookies.add_private(cookie);
        Ok(())
    }

    /// Creates a new user from a form or a json. The user will not be authenticated by default.
    /// In order to authenticate the user, cast the signup form to a login form or use `signup_for`.
    /// ```rust
    /// # use rocket::{post, form::Form};
    /// # use rocket_auth_nosql::{Auth, Signup, Error};
    /// # use std::time::Duration;
    /// #[post("/signup", data="<form>")]
    /// async fn signup(form: Form<Signup>, auth: Auth<'_>) -> Result<&'static str, Error>{
    ///     auth.signup(&form).await?;
    ///     auth.login(&form.into()).await?;
    ///     Ok("Logged in")
    /// }
    /// ```

    pub async fn signup(&self, form: &Signup) -> Result<()>  {
        self.users.signup(form).await;
        Ok(())
    }

    /// Creates a new user from a form or a json.
    /// The session will last the specified period of time.
    /// ```rust
    /// # use rocket::{post, form::Form};
    /// # use rocket_auth_nosql::{Auth, Signup};
    /// # use std::time::Duration;
    /// #[post("/signup", data="<form>")]
    /// fn signup_for(form: Form<Signup>, auth: Auth) {
    ///     let one_hour = Duration::from_secs(60 * 60);
    ///     auth.signup_for(&form, one_hour);
    /// }
    /// ```

    pub async fn signup_for(&self, form: &Signup, time: Duration) -> Result<()>  {
        self.users.signup(form).await;
        self.login_for(&form.clone().into(), time).await?;
        Ok(())
    }

    ///
    ///
    /// It allows to know if the current client is authenticated or not.
    /// ```rust
    /// # use rocket::{get};
    /// # use rocket_auth_nosql::{Auth};
    /// #[get("/am-I-authenticated")]
    /// fn is_auth(auth: Auth<'_>) -> &'static str {
    ///     if auth.is_auth() {
    ///         "Yes you are."
    ///     } else {
    ///         "nope."
    ///     }
    /// }
    /// # fn main() {}
    /// ```
    pub fn is_auth(&self) -> bool {
        if let Some(session) = &self.session {
            self.users.is_auth(session)
        } else {
            false
        }
    }

    /// It retrieves the current logged user.  
    /// ```
    /// # use rocket::get;
    /// # use rocket_auth_nosql::Auth;
    /// #[get("/display-me")]
    /// async fn display_me(auth: Auth<'_>) -> String {
    ///     format!("{:?}", auth.get_user().await)
    /// }
    /// ```
    pub async fn get_user(&self) -> Option<User> {
        if !self.is_auth() {
            return None;
        }
        let id = self.session.as_ref()?.id;
        if let Ok(user) = self.users.get_by_id(id).await {
            Some(user)
        } else {
            None
        }
    }
    /// Logs the currently authenticated user out.
    /// ```rust
    /// # use rocket::get;
    /// # use rocket_auth_nosql::Auth;
    /// #[get("/logout")]
    /// fn logout(auth: Auth)  {
    ///     auth.logout();
    /// }
    /// ```

    pub fn logout(&self) -> Result<()>  {
        let session = self.get_session()?;
        self.users.logout(session);
        self.cookies.remove_private(Cookie::named("rocket_auth_nosql"));
        Ok(())
    }
    /// Deletes the account of the currently authenticated user.
    /// ```rust
    /// # use rocket::get;
    /// # use rocket_auth_nosql::Auth;
    /// #[get("/delete-my-account")]
    /// fn delete(auth: Auth)  {
    ///     auth.delete();
    /// }
    /// ```

    pub async fn delete(&self)-> Result<()>  {
        if self.is_auth() {
            let session = self.get_session()?;
            self.users.delete(session.id).await;
            self.cookies.remove_private(Cookie::named("rocket_auth_nosql"));
            Ok(())
        } else {
            Err(Error::UnauthenticatedError)
        }
    }

    /// Changes the password of the currently authenticated user
    /// ```
    /// # use rocket_auth_nosql::Auth;
    /// # use rocket::post;
    /// # #[post("/change")]
    /// # fn example(auth: Auth<'_>) {
    ///     auth.change_password("new password");
    /// # }
    /// ```

    pub async fn change_password(&self, password: &str) -> Result<()>  {
        if self.is_auth() {
            let session = self.get_session()?;
            let mut user = self.users.get_by_id(session.id).await?;
            user.set_password(password)?;
            self.users.modify(&user).await;
            Ok(())
        } else {
            Err(Error::UnauthorizedError)
        }
    }
    /// Changes the password of the currently authenticated user
    /// ```
    /// # use rocket_auth_nosql::Auth;
    /// # use rocket::post;
    /// # #[post("/change")]
    /// # fn example(auth: Auth<'_>) {
    ///     auth.change_password("new password");
    /// # }
    /// ```

    pub async fn verify_account(&self, token: &str) -> Result<()>  {
        if self.is_auth() {
            let session = self.get_session()?;
            let mut user = self.users.get_by_id(session.id).await?;
            user.set_verified(token)?;
            self.users.modify(&user).await;
            Ok(())
        } else {
            Err(Error::VerificationTokenMismatch)
        }
    }
    /// Changes the email of the currently authenticated user
    /// ```
    /// # use rocket_auth_nosql::Auth;
    /// # fn func(auth: Auth) {
    /// auth.change_email("new@email.com".into());
    /// # }
    /// ```

    pub async fn change_email(&self, email: String) -> Result<()>  {
        if self.is_auth() {
            if !validator::validate_email(&email) {
                return Err(Error::InvalidEmailAddressError);
            }
            let session = self.get_session()?;
            let mut user = self.users.get_by_id(session.id).await?;
            user.email = email;
            self.users.modify(&user).await;
            Ok(())
        } else {
            Err(Error::UnauthorizedError)
        }
    }

    /// This method is useful when the function returns a Result type.
    /// It is intended to be used primarily
    /// with the `?` operator.
    /// ```
    /// # fn func(auth: rocket_auth_nosql::Auth) -> Result<(), rocket_auth_nosql::Error> {
    /// auth.get_session()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_session(&self) -> Result<&Session> {
        let session = self.session.as_ref().ok_or(Error::UnauthenticatedError)?;
        Ok(session)
    }
}
