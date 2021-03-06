use crate::db::Conn;
use crate::models::user::{User, UserList};
use crate::schema::users;
use std::ops::Deref;

use crypto::scrypt::{scrypt_check, scrypt_simple, ScryptParams};
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error};
use serde::Deserialize;

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub hash: &'a str,
}

pub enum UserCreationError {
    DuplicatedEmail,
    DuplicatedUsername,
}

impl From<Error> for UserCreationError {
    fn from(err: Error) -> UserCreationError {
        if let Error::DatabaseError(DatabaseErrorKind::UniqueViolation, info) = &err {
            match info.constraint_name() {
                Some("users_username_key") => return UserCreationError::DuplicatedUsername,
                Some("users_email_key") => return UserCreationError::DuplicatedEmail,
                _ => {}
            }
        }
        panic!("Error creating user: {:?}", err)
    }
}

pub fn create(
    conn: &Conn,
    username: &str,
    email: &str,
    password: &str,
) -> Result<User, UserCreationError> {
    // see https://blog.filippo.io/the-scrypt-parameters
    let hash = &scrypt_simple(password, &ScryptParams::new(14, 8, 1)).expect("hash error");

    let new_user = &NewUser {
        username,
        email,
        hash,
    };

    diesel::insert_into(users::table)
        .values(new_user)
        .get_result::<User>(conn.deref())
        .map_err(Into::into)
}

pub fn login(conn: &Conn, email: &str, password: &str) -> Option<User> {
    let user = users::table
        .filter(users::email.eq(email))
        .get_result::<User>(conn.deref())
        .map_err(|err| eprintln!("login_user: {}", err))
        .ok()?;

    let password_matches = scrypt_check(password, &user.hash)
        .map_err(|err| eprintln!("login_user: scrypt_check: {}", err))
        .ok()?;

    if password_matches {
        Some(user)
    } else {
        eprintln!(
            "login attempt for '{}' failed: password doesn't match",
            email
        );
        None
    }
}

pub fn find(conn: &Conn) -> Option<UserList> {

    let users : Vec<User> = users::table.load::<User>(conn.deref())
        .map_err(|err| println!("Can not load users!: {}", err))
        .unwrap();

    Some(UserList{
        users
    })
}

pub fn find_one(conn: &Conn, id: i32) -> Option<User> {
    users::table
        .find(id)
        .get_result(conn.deref())
        .map_err(|err| println!("find_user: {}", err))
        .ok()
}

pub fn delete(conn: &Conn, id: i32) {
    let result = diesel::delete(users::table.filter(users::id.eq(id))).execute(conn.deref());
    if let Err(err) = result {
        eprintln!("users::delete: {}", err);
    }
}

// TODO: remove clone when diesel will allow skipping fields
#[derive(Deserialize, AsChangeset, Default, Clone)]
#[table_name = "users"]
pub struct UpdateUserData {
    username: Option<String>,
    email: Option<String>,
    bio: Option<String>,
    image: Option<String>,

    // hack to skip the field
    #[column_name = "hash"]
    password: Option<String>,
}

pub fn update(conn: &Conn, id: i32, data: &UpdateUserData) -> Option<User> {
    let data = &UpdateUserData {
        password: None,
        ..data.clone()
    };
    diesel::update(users::table.find(id))
        .set(data)
        .get_result(conn.deref())
        .ok()
}
