use rocket::{form::*, get, post, response::Redirect, routes, State};
use rocket_auth_nosql::{prelude::Error, *};
use rocket_dyn_templates::Template;
use serde_json::json;

use std::result::Result;
use std::*;

const DATABASE: &str = "rocket_auth_nosql";

#[get("/login")]
fn get_login() -> Template {
    Template::render("login", json!({}))
}

#[post("/login", data = "<form>")]
async fn post_login(auth: Auth<'_>, form: Form<Login>) -> Result<Redirect, Error> {
    let result = auth.login(&form).await;
    println!("login attempt: {:?}", result);
    result?;
    Ok(Redirect::to("/"))
}

#[get("/signup")]
async fn get_signup() -> Template {
    Template::render("signup", json!({}))
}

#[post("/signup", data = "<form>")]
async fn post_signup(auth: Auth<'_>, form: Form<Signup>) -> Result<Redirect, Error> {
    auth.signup(&form).await?;
    auth.login(&form.into()).await?;

    Ok(Redirect::to("/"))
}

#[get("/")]
async fn index(user: Option<User>) -> Template {
    Template::render("index", json!({ "user": user }))
}

#[get("/logout")]
fn logout(auth: Auth<'_>) -> Result<Template, Error> {
    auth.logout()?;
    Ok(Template::render("logout", json!({})))
}
// #[get("/activate/?<email>&<token>")]
// fn get_activate(email: &str, token: &str) -> Result<Template, Error> {

//     Ok(Template::render("activate", json!({})))
// }
#[get("/delete")]
async fn delete(auth: Auth<'_>) -> Result<Template, Error> {
    auth.delete().await?;
    Ok(Template::render("deleted", json!({})))
}

#[get("/show_all_users")]
async fn show_all_users(auth: Auth<'_>, conn: &State<Users>, user: Option<User>) -> Result<Template, Error> {
    if auth.is_auth() {
        let users: Vec<User> = conn.get_all_users().await?;
        println!("{:?}", users);
        Ok(Template::render(
            "users",
            json!({"users": users, "user": user}),
        ))
    } else {
        Ok(Template::render("index", json!({ "user": user })))
    }
}

#[rocket::main]
async fn main() -> Result<(), Error> {
    let users = Users::open_mongodb("mongodb://localhost:27017", DATABASE).await?;

    let _result = rocket::build()
        .mount(
            "/",
            routes![
                index,
                get_login,
                post_signup,
                get_signup,
                //get_activate,
                post_login,
                logout,
                delete,
                show_all_users
            ],
        )
        .manage(users)
        .attach(Template::fairing())
        .launch()
        .await
        .unwrap();
        Ok(())
        
}
