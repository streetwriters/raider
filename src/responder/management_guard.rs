// Raider
//
// Affiliates dashboard
// Copyright: 2021, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::APP_CONF;
use base64;
use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest, Request};

gen_auth_guard!(ManagementGuard, APP_CONF.server.management_token);
