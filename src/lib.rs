//! rocket_auth_nosql provides a ready-to-use NoSQL backed API for authentication management.
//! It lets you create, delete, and authenticate users.
//! The available features are:
//! * `redis`: for storing sessions on a redis server using `redis`.
//! * `mongodb`: for interacting with a MongoDB database using `mongodb`.
//!
//!
//! `rocket_auth_nosql` uses private cookies to store session data.
//! This means that in order for cookies to be properly decrypted between launches, a `secret_key` must be set.
//! For more information visit rocket's [configuration guide](https://rocket.rs/v0.5-rc/guide/configuration/#configuration).
//!
//!
//!
//! To use `rocket_auth_nosql` include it as a dependency in your Cargo.toml file:
//! ```
//! [dependencies.rocket_auth_nosql]
//! version = "0.0.1"
//! ```
//! # Quick overview
//! This crate provides three guards:
//! * [`Auth`]: Manages authentication.
//! * [`Session`]: Used to retrieve session data from client cookies.
//! * [`User`]: Restricts content, so it can be viewed by authenticated clients only.
//!
//!
//! It also includes two structs to be parsed from forms and json data:
//! * [`Signup`]: Used to create new users.
//! * [`Login`]: Used to authenticate users.
//!
//!
//! Finally it has two structures for queries:
//! * [`Users`]: It allows to query users to the database.
//! * [`User`]: It is the response of a query.
//!
//!
//! ## Auth guard
//! The [`Auth`] guard allows to log in, log out, sign up, modify, and delete the currently (un)authenticated user.
//! For more information see [`Auth`]. Because of Rust's ownership rules, you may not retrieve both [`rocket::http::CookieJar`] and the [`Auth`] guard
//! simultaneously. However, retrieving cookies is not needed since `Auth` stores them in the public field [`Auth::cookies`].
//!  A working example:
//! ```rust,no_run
//! use rocket::{get, post, form::Form, routes};
//! use rocket_auth_nosql::{Users, Error, Auth, Signup, Login};
//!
//! #[post("/signup", data="<form>")]
//! async fn signup(form: Form<Signup>, auth: Auth<'_>) -> Result<&'static str, Error> {
//!     auth.signup(&form).await?;
//!     auth.login(&form.into());
//!     Ok("You signed up.")
//! }
//!
//! #[post("/login", data="<form>")]
//! async fn login(form: rocket::serde::json::Json<Login>, auth: Auth<'_>) -> Result<&'static str, Error> {
//!     auth.login(&form).await?;
//!     Ok("You're logged in.")
//! }
//!
//! #[get("/logout")]
//! fn logout(auth: Auth<'_>) {
//!     auth.logout();
//! }
//! #[tokio::main]
//! async fn main() -> Result<(), Error>{
//!     let users = Users::open_sqlite("mydb.db").await?;
//!
//!     rocket::build()
//!         .mount("/", routes![signup, login, logout])
//!         .manage(users)
//!         .launch();
//!     Ok(())
//! }
//! ```
//!
//! ## Users struct
//! The [`Users`] struct administers interactions with the database.
//! It lets you query, create, modify and delete users.
//! Unlike the [`Auth`] guard, a [`Users`] instance can manage any user in the database.
//! Note that the [`Auth`] guards includes a `Users` instance stored on the public `users` field.
//! So it is not necessary to retrieve Users when using `Auth`.
//! A simple example of how to query a user with the [`Users`] struct:
//!
//! ```rust
//! # use rocket::{get, State};
//! # use serde_json::json;
//! use rocket_auth_nosql::Users;
//!
//! #[get("/see-user/<id>")]
//! async fn see_user(id: i32, users: &State<Users>) -> String {
//!     let user = users.get_by_id(id).await.unwrap();
//!     format!("{}", json!(user))
//! }
//! # fn main() {}
//! ```
//!
//! A [`Users`] instance can be constructed by connecting it to the database with the methods [`open_sqlite`](Users::open_sqlite),
//! [`open_postgres`](Users::open_postgres) or [`open_rusqlite`](Users::open_rusqlite). Furthermore, it can be constructed from a working connection.
//!
//!
//! ## User guard
//! The [`User`] guard can be used to restrict content so it can only be viewed by authenticated users.
//! Additionally, you can use it to render special content if the client is authenticated or not.
//! ```rust
//! # use rocket::*;
//! # use rocket_auth_nosql::User;
//! #[get("/private-content")]
//! fn private_content(user: User) -> &'static str {
//!     "If you can see this, you are logged in."
//! }
//!
//! #[get("/special-content")]
//! fn special_content(option: Option<User>) -> String {
//!     if let Some(user) = option {
//!         format!("hello, {}.", user.email())
//!     } else {
//!         "hello, anonymous user".into()
//!     }
//! }
//! ```
//!
//! ## AdminUser guard
//! The [`AdminUser`] guard can be used analogously to [`User`].
//! It will restrict content so it can be viewed by admins only.
//! ```
//! # use rocket::*;
//! # use rocket_auth_nosql::AdminUser;
//! #[get("/admin-panel")]
//! fn admin_panel(user: AdminUser) -> String {
//!    format!("Hello {}.", user.email())
//! }
//! ```


mod cookies;
mod db;
mod error;
mod forms;
pub mod prelude;
mod session;
mod user;

#[cfg(test)]
mod tests;

use std::fmt::Debug;

pub use prelude::*;

pub use crate::user::auth::Auth;
pub use cookies::Session;
pub use error::Error;
use mongodb::bson::{oid::ObjectId};

/// The `User` guard can be used to restrict content so it can only be viewed by authenticated users.
/// ```rust
/// #
/// # use rocket::{get};
/// # use rocket_auth_nosql::User;
/// #[get("/private-content")]
/// fn private_content(user: User) -> &'static str {
///     "If you can see this, you are logged in."
/// }
/// # fn main() {}
/// ```
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub struct User {
	#[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    email: String,
    pub is_admin: bool,
    is_verified: bool,
    verification_token: String,
    password: String,
    prev_password: Option<String>,
    prev_password_1: Option<String>,
    prev_password_2: Option<String>,
}

/// The [`AdminUser`] guard can be used analogously to [`User`].
/// It will restrict content so it can be viewed by admins only.
/// ```
/// # use rocket::*;
/// # use rocket_auth_nosql::AdminUser;
/// #[get("/admin-panel")]
/// fn admin_panel(user: AdminUser) -> String {
///    format!("Hello {}.", user.email())
/// }
/// ```
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub struct AdminUser(User);

/// The [`UnverifiedUser`] guard can be used analogously to [`User`].
/// It is restricted to content for users that have authenticated
/// but have not yet verified their email address.
/// ```
/// # use rocket::*;
/// # use rocket_auth_nosql::UnverifiedUser;
/// #[get("/newuser-panel")]
/// fn newuser_panel(user: UnverifiedUser) -> String {
///    format!("Hello {}.", user.email())
/// }
/// ```
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub struct UnverifiedUser(User);

impl Debug for AdminUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Admin{:?}", self.0)
    }
}

/// The `Users` struct is used to query users from the database, as well as to create, modify and delete them.
pub struct Users {
    conn: Box<dyn DBConnection>,
    sess: Box<dyn SessionManager>,
}
