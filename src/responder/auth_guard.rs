// Raider
//
// Affiliates dashboard
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use log;
use rand::{self, Rng};
use rocket::http::{Cookie, CookieJar, Status};
use rocket::request::Outcome;
use rocket::request::{self, FromRequest, Request};
use sha2::{Digest, Sha256};

use crate::APP_CONF;

pub struct AuthGuard(pub i32);
pub struct AuthAnonymousGuard;

const PASSWORD_MINIMUM_LENGTH: usize = 4;
const PASSWORD_MAXIMUM_LENGTH: usize = 200;

pub static AUTH_USER_COOKIE_NAME: &'static str = "user_id";

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthGuard {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<AuthGuard, ()> {
        if let Outcome::Success(cookies) = request.guard::<&CookieJar>().await {
            if let Some(user_id_cookie) = read(cookies.clone()) {
                if let Ok(user_id) = user_id_cookie.value().parse::<i32>() {
                    log::debug!("got user_id from cookies: {}", &user_id);

                    return Outcome::Success(AuthGuard(user_id));
                }
            }
        }

        Outcome::Error((Status::Forbidden, ()))
    }
}

#[rocket::async_trait]
impl<'a> FromRequest<'a> for AuthAnonymousGuard {
    type Error = ();

    async fn from_request(request: &'a Request<'_>) -> request::Outcome<AuthAnonymousGuard, ()> {
        match request.guard::<AuthGuard>().await {
            Outcome::Success(_) => Outcome::Error((Status::Gone, ())),
            _ => Outcome::Success(AuthAnonymousGuard),
        }
    }
}

pub fn insert(cookies: &CookieJar, user_id: String) {
    cookies.add_private(Cookie::new(AUTH_USER_COOKIE_NAME, user_id));
}

pub fn cleanup(cookies: CookieJar) {
    cookies.remove_private(Cookie::named(AUTH_USER_COOKIE_NAME));
}

fn read(cookies: CookieJar) -> Option<Cookie> {
    cookies.get_private(AUTH_USER_COOKIE_NAME)
}

pub fn password_encode(password: &str) -> Vec<u8> {
    let password_salted = [password, APP_CONF.database.password_salt.as_str()].join("");

    log::debug!(
        "salted password: {} and got result: {}",
        password,
        &password_salted
    );

    let mut hasher = Sha256::default();

    hasher.input(&password_salted.into_bytes());

    hasher.result().to_vec()
}

pub fn password_verify(reference: &[u8], password: &str) -> bool {
    let password_encoded = password_encode(password);

    password_encoded == reference
}

pub fn password_generate() -> (Vec<u8>, String) {
    let password = rand::thread_rng()
        .gen_ascii_chars()
        .take(60)
        .collect::<String>();

    (password_encode(&password), password)
}

pub fn recovery_generate() -> (Vec<u8>, String) {
    let recovery_password = rand::thread_rng()
        .gen_ascii_chars()
        .take(40)
        .collect::<String>();

    (password_encode(&recovery_password), recovery_password)
}

pub fn password_policy_check(password: &str) -> bool {
    let size = password.len();

    size >= PASSWORD_MINIMUM_LENGTH && size <= PASSWORD_MAXIMUM_LENGTH
}
