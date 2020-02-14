#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
use rocket_cors;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate validator_derive;

mod auth;
pub mod config;
mod db;
mod errors;
mod models;
mod routes;
mod schema;

use rocket_contrib::json::JsonValue;
use rocket_cors::Cors;
use dotenv;

#[catch(404)]
fn not_found() -> JsonValue {
    json!({
        "status": "error",
        "reason": "Resource was not found."
    })
}

pub enum Environment{
    Dev,
    Test,
    Production
}

fn cors_fairing() -> Cors {
    Cors::from_options(&Default::default()).expect("Cors fairing cannot be created")
}

pub fn rocket(env: Option<Environment>) -> rocket::Rocket {

    // Load proper .env file
    match env {
        Some(e) => dotenv::from_filename(".env.test").ok(),
        _ => dotenv::dotenv().ok()
    };

    rocket::custom(config::from_env())
        .mount(
            "/api",
            routes![
                routes::users::post_users,
                routes::users::post_users_login,
                routes::users::put_user,
                routes::users::get_user,
                routes::users::get_me,
                routes::users::delete_user,
            ],
        )
        .attach(db::Conn::fairing())
        .attach(cors_fairing())
        .attach(config::AppState::manage())
        .register(catchers![not_found])
}
