//! This file contains utility functions used by all tests.
#![allow(unused)]
use rocket::http::{ContentType, Header, Status};
use rocket::local::{Client, LocalResponse};
use serde_json::Value;
use once_cell::sync::OnceCell;
use g_users::config::TOKEN_PREFIX;
use g_users::{Environment, load_env};
use std::process::{Command, Stdio};
use std::ops::Deref;
use std::env;
use std::fs::File;
use run_script::ScriptOptions;

extern crate run_script;

use run_script::run_or_exit;


pub const USERNAME: &'static str = "tuser";
pub const EMAIL: &'static str = "tuser@example.io";
pub const PASSWORD: &'static str = "mustbe8ormore";





/// Utility macro for turning `json!` into string.
#[macro_export]
macro_rules! json_string {
    ($value:tt) => {
        serde_json::to_string(&serde_json::json!($value)).expect("cannot json stringify")
    };
}


pub type Token = String;


pub fn setup_db(){

    load_env(Some(Environment::Test));
    let database_url =
        env::var("DATABASE_URL").expect("No DATABASE_URL environment variable found");

    // Create test database using env var ( from .env.test ) if not exists,
    // by calling "diesel reset". Only the database is different, migration files are the same.
    // We also cleans the old test database on each call. "diesel database reset" first cleans the
    // old database and than sets up the new one again.
    let options = ScriptOptions::new();
    let args = vec!["--database-url".to_string(), database_url];
    let (output, error) = run_script::run_or_exit(
        r#"
        diesel database reset
        "#,
        &args,
        &options
    );

    println!("Output: {}", output);
    println!("Error: {}", error);
}



pub fn test_client() -> &'static Client {
    static INSTANCE: OnceCell<Client> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        setup_db();
        let rocket = g_users::rocket();
        Client::new(rocket).expect("valid rocket instance")
    })

}


/// Retrieve a token registering a user if required.
pub fn login(client: &Client) -> Token {
    try_login(client).unwrap_or_else(|| {
        register(client, USERNAME, EMAIL, PASSWORD);
        try_login(client).expect("Cannot login")
    })
}


/// Make an authorization header.
pub fn token_header(token: Token) -> Header<'static> {
    Header::new("authorization", format!("{}{}", TOKEN_PREFIX, token))
}


/// Helper function for converting response to json value.
pub fn response_json_value(response: &mut LocalResponse) -> Value {
    let body = response.body().expect("no body");
    serde_json::from_reader(body.into_inner()).expect("can't parse value")
}


// Internal stuff


/// Login as default user returning None if login is not found
fn try_login(client: &Client) -> Option<Token> {
    let response = &mut client
        .post("/api/users/login")
        .header(ContentType::JSON)
        .body(json_string!({"user": {"email": EMAIL, "password": PASSWORD}}))
        .dispatch();

    if response.status() == Status::UnprocessableEntity {
        return None;
    }

    let value = response_json_value(response);
    let token = value
        .get("user")
        .and_then(|user| user.get("token"))
        .and_then(|token| token.as_str())
        .map(String::from)
        .expect("Cannot extract token");
    Some(token)
}


/// Register user for
pub fn register(client: &Client, username: &str, email: &str, password: &str) {
    let response = client
        .post("/api/users")
        .header(ContentType::JSON)
        .body(json_string!({"user": {"username": username, "email": email, "password": password}}))
        .dispatch();

    match response.status() {
        Status::Ok | Status::UnprocessableEntity => {} // ok,
        status => panic!("Registration failed: {}", status)
    }
}
