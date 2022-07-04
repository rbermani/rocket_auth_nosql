# rocket_auth_nosql
rocket_auth_nosql provides an API for NoSQL based authentication management.
It uses MongoDB for persistence. It lets you create, delete, and authenticate users.
It is derived from a hard fork of the rocket_auth project by Tomás Vallotton
The available features are:
* `redis`: for storing sessions on a redis server using `redis`.

`rocket_auth_nosql` uses private cookies to store session data.
This means that in order for cookies to be properly decrypted between launches, a `secret_key` must be set.
For more information visit rocket's [configuration guide](https://rocket.rs/v0.5-rc/guide/configuration/#configuration).

# New Features
- [x] Replace backend with MongoDB
- [x] Refactoring to use more idiomatic Rust
- [x] Add UnverifiedUser guard

# TODO
- [ ] Add CSRF Protection
- [ ] Add E-Mail content templating
- [ ] Add TOML file configuration capability for server information, credentials, etc
- [ ] Ajax inline form responses
- [ ] Add E-mail logic
- [ ] Protected selection criteria schema change, so password field is only returned on explicit queries
- [ ] Forgotten password reset mechanism
- [ ] New User sign-up email verification
- [ ] Prevent re-use of previous password during validation
- [ ] Unit tests



To use `rocket_auth_nosql` include it as a dependency in your Cargo.toml file:
```ini
[dependencies.rocket_auth_nosql]
version = "0.0.1"
```
# Quick overview
This crate provides five guards:
* [`Auth`]: Manages authentication.
* [`Session`]: Used to retrieve session data from client cookies.
* [`UnverifiedUser`]: Succeeds if the user is authenticated, but has not yet verified their account.
* [`User`]: Restricted content, so it can be viewed by authenticated and verified clients only.
* [`AdminUser`]: Restricts Controls, that can only be used by privileged, authenticated, and verified users with the administrator flag set.

It also includes two structs to be parsed from forms and json data:
* [`Signup`]: Used to create new users.
* [`Login`]: Used to authenticate users.

Finally it has two structures for queries:
* [`Users`]: It allows to query users to the database.
* [`User`]: It is the response of a query.
## Auth guard
The [`Auth`] guard allows to log in, log out, sign up, modify, and delete the currently (un)authenticated user.
For more information see [`Auth`]. Because of Rust's ownership rules, you may not retrieve both [`rocket::http::CookieJar`] and the [`Auth`] guard
simultaneously. However, retrieving cookies is not needed since `Auth` stores them in the public field [`Auth::cookies`].
A working example:
```rust
use rocket::{get, post, form::Form, routes};
use rocket_auth_nosql::{Users, Error, Auth, Signup, Login};

#[post("/signup", data="<form>")]
async fn signup(form: Form<Signup>, auth: Auth<'_>) -> Result<&'static str, Error> {
    auth.signup(&form).await?;
    auth.login(&form.into());
    Ok("You signed up.")
}

#[post("/login", data="<form>")]
async fn login(form: Form<Login>, auth: Auth<'_>) -> Result<&'static str, Error>{
    auth.login(&form).await?;
    Ok("You're logged in.")
}

#[get("/logout")]
fn logout(auth: Auth<'_>) {
    auth.logout();
}
#[tokio::main]
async fn main() -> Result<(), Error>{
    let users = Users::open_mongodb("mongdb://localhost:27017", DATABASE).await?;

    rocket::build()
        .mount("/", routes![signup, login, logout])
        .manage(users)
        .launch();
    Ok(())
}
```

## Users struct
The `Users` struct administers interactions with the database.
It lets you query, create, modify and delete users.
Unlike the `Auth` guard, a `Users` instance can manage any user in the database.
Note that the `Auth` guards includes a `Users` instance stored on the public `users` field.
So it is not necessary to retrieve Users when using `Auth`.
A simple example of how to query a user with the `Users` struct:

```rust
use rocket_auth_nosql::Users;

#[get("/see-user/<id>")]
async fn see_user(id: i32, users: &State<Users>) -> String {
    let user = users.get_by_id(id).await.unwrap();
    format!("{}", json!(user))
}
```

A `Users` instance can be constructed by connecting it to the database with the methods `open_sqlite`,
`open_postgres`. Furthermore, it can be constructed from a working connection.


## User guard
The `User` guard can be used to restrict content so it can only be viewed by authenticated users.
Additionally, you can use it to render special content if the client is authenticated or not.
```rust
#[get("/private-content")]
fn private_content(user: User) -> &'static str {
    "If you can see this, you are logged in."
}

#[get("/special-content")]
fn special_content(option: Option<User>) -> String {
    if let Some(user) = option {
        format!("hello, {}.", user.email())
    } else {
        "hello, anonymous user".into()
    }
}
#[get("/admins-only")]
fn admins_only(user: AdminUser) -> &'static str {
   "Hello administrator."
}
```

## AdminUser guard
The `AdminUser` guard can be used analogously to `User`.
It will restrict content so it can be viewed by admins only.
```rust
# use rocket::*;
# use rocket_auth_nosql::AdminUser;
#[get("/admin-panel")]
fn admin_panel(user: AdminUser) -> String {
   format!("Hello {}.", user.email());
}
```
