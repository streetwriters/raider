// Raider
//
// Affiliates dashboard
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use log;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Outcome};
use rocket::{Request, State};
use std::ops::{Deref, DerefMut};
use std::time::Duration;

use crate::APP_CONF;

type Pool = diesel::r2d2::Pool<ConnectionManager<SqliteConnection>>;

pub struct DbConn(pub r2d2::PooledConnection<ConnectionManager<SqliteConnection>>);

impl Deref for DbConn {
    type Target = SqliteConnection;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DbConn {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[rocket::async_trait]
impl<'a> FromRequest<'a> for DbConn {
    type Error = ();

    async fn from_request(request: &'a Request<'_>) -> request::Outcome<DbConn, ()> {
        let pool = request.guard::<&State<Pool>>().await.unwrap();
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Error((Status::ServiceUnavailable, ())),
        }
    }
}

pub fn pool() -> Pool {
    log::debug!("setting up db pool...");

    let manager = ConnectionManager::<SqliteConnection>::new(APP_CONF.database.url.as_str());

    let pool = r2d2::Pool::builder()
        .max_size(APP_CONF.database.pool_size)
        .idle_timeout(Some(Duration::from_secs(APP_CONF.database.idle_timeout)))
        .connection_timeout(Duration::from_secs(APP_CONF.database.connection_timeout))
        .build(manager)
        .expect("db pool");

    log::debug!("db pool configured");

    pool
}
