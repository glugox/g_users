//! This file contains utility functions used by all tests.
#![allow(unused)]
use g_users::config::TOKEN_PREFIX;
use g_users::{load_env, Environment};
use g_users::models::user::User;
use once_cell::sync::OnceCell;
use rocket::http::{ContentType, Header, Status};
use rocket::local::{Client, LocalResponse};
use fake::{Dummy, Fake, Faker};
use run_script::ScriptOptions;
use serde_json::Value;
use std::env;
use std::fs::File;
use std::ops::Deref;
use std::process::{Command, Stdio};
use rand::{thread_rng, Rng};
use rand::distributions::{Alphanumeric};


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


#[derive(Debug, Dummy)]
pub struct UserData {
    pub username: String,
    pub email: String,
    pub password: String
}

impl Default for UserData {
    fn default() -> UserData{
        UserData{
            username: USERNAME.parse().unwrap(),
            password: PASSWORD.parse().unwrap(),
            email: EMAIL.parse().unwrap()
        }
    }
}

pub fn setup_db() {
    load_env(Some(Environment::Test));
    let database_url =
        env::var("DATABASE_URL").expect("No DATABASE_URL environment variable found");

    // Create test database using env var ( from .env.test ) if not exists,
    // by calling "diesel reset". Only the database is different, migration files are the same.
    // We also cleans the old test database on each call. "diesel database reset" first cleans the
    // old database and than sets up the new one again.
    let options = ScriptOptions::new();
    let args = vec!["--database-url".to_string(), database_url];
    let (code, output, error) = run_script::run(
        r#"
        diesel database reset
        "#,
        &args,
        &options,
    ).unwrap();

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

/// Make an authorization header with api key.
pub fn api_header(token: Token) -> Header<'static> {
    Header::new("x-api-key", format!("{}", token))
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
pub fn register(client: &Client, username: &str, email: &str, password: &str) -> Option<u64> {



    let response = &mut client
        .post("/api/users")
        .header(ContentType::JSON)
        .body(json_string!({"user": {"username": username, "email": email, "password": password}}))
        .dispatch();

    let value = response_json_value(response);
    let id = value
        .get("user")
        .and_then(|user| user.get("id"));
    let id_val = id.expect("Can't extract ID");
    let id_str = id_val.as_u64();

    Option::from(match response.status() {
        Status::Ok | Status::UnprocessableEntity => id_str.unwrap(), // ok,
        status => panic!("Registration failed: {}", status),
    })
}


pub fn generate_random_user_data() -> UserData {
    let mut rand: String = thread_rng().sample_iter(&Alphanumeric).take(9).collect();
    // Avoid starting it with number
    rand = String::from("a") + &rand;
    UserData {
        username: rand.to_owned(),
        email: rand.to_owned() + "@example.com",
        password: rand.to_owned()
    }
}