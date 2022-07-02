use rocket::{form::*, get, post, response::Redirect, routes};
use rocket_auth_nosql::{prelude::Error, *};
use rocket_dyn_templates::Template;
use serde_json::json;

use std::result::Result;
use std::*;

const DATABASE: &str = "rocket_auth_nosql";

#[get("/activate?<token>")]
async fn get_activate(token: &str, auth: Auth<'_>, _user: UnverifiedUser) -> Result<Template, Error> {
    auth.verify_account(token).await?;
    Ok(Template::render("activate", json!({})))
}

#[get("/")]
async fn index(user: User) -> Template {
    Template::render("index", json!({ "user": user }))
}

#[get("/", rank = 2)]
async fn index_unverified(user: UnverifiedUser) -> Template {
    Template::render("index", json!({ "user": user }))
}

#[get("/", rank = 3)]
async fn index_redirect() -> Result<Redirect, Error>  {
    Ok(Redirect::to("/login"))
}

#[get("/admin")]
fn get_admin_panel(admin: AdminUser) -> Template {
    Template::render("admin", json!({"admin":admin}))
}

#[get("/admin", rank = 2)]
fn admin_panel_reject(user: User) -> Template {
    Template::render("admin_reject", json!({"user": user}))
}

// Admin catch-all route
#[get("/admin", rank = 3)]
fn admin_panel_redirect() -> Result<Redirect, Error>  {
    Ok(Redirect::to("/login"))
}

#[post("/login", data = "<form>")]
async fn post_login(auth: Auth<'_>, form: Form<Login>) -> Result<Redirect, Error> {
    let result = auth.login(&form).await;
    println!("login attempt: {:?}", result);
    result?;
    Ok(Redirect::to("/"))
}

#[get("/login", rank = 2)]
fn get_login() -> Template {
    Template::render("login", json!({}))
}

#[get("/signup")]
async fn get_signup() -> Template {
    Template::render("signup", json!({}))
}

#[post("/signup", data = "<form>")]
async fn post_signup(auth: Auth<'_>, form: Form<Signup>) -> Result<Redirect, Error> {
    auth.signup(&form).await?;
    auth.login(&form.into()).await?;

    Ok(Redirect::to("/login"))
}

#[get("/logout")]
fn logout(auth: Auth<'_>) -> Result<Template, Error> {
    auth.logout()?;
    Ok(Template::render("logout", json!({})))
}

#[get("/delete")]
async fn delete(auth: Auth<'_>) -> Result<Template, Error> {
    auth.delete().await?;
    Ok(Template::render("deleted", json!({})))
}

#[get("/show_all_users")]
async fn show_all_users(auth: Auth<'_>, user: User) -> Result<Template, Error> {
    let users: Vec<User> = auth.users.get_all_users().await?;
    println!("{:?}", users);
    Ok(Template::render(
        "users",
        json!({"users": users, "user": user}),
    ))
}

#[get("/show_all_users", rank = 2)]
async fn show_all_users_reject() -> Result<Template, Error> {
    Ok(Template::render("index", json!({ })))
}

#[rocket::main]
async fn main() -> Result<(), Error> {
    let users = Users::open_mongodb("mongodb://localhost:27017", DATABASE).await?;

    let _result = rocket::build()
        .mount(
            "/",
            routes![
                index,
                index_unverified,
                index_redirect,
                get_admin_panel,
                admin_panel_reject,
                admin_panel_redirect,
                get_login,
                post_signup,
                get_signup,
                get_activate,
                post_login,
                logout,
                delete,
                show_all_users,
                show_all_users_reject
            ],
        )
        .manage(users)
        .attach(Template::fairing())
        .launch()
        .await
        .unwrap();
        Ok(())
        
}
