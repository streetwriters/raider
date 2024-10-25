// Raider
//
// Affiliates dashboard
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::{catchers, routes};
use crate::storage::db;
use rocket::{
    config::{Config, SecretKey},
    fairing::AdHoc,
    Build, Rocket,
};
use rocket_dyn_templates::Template;

use crate::APP_CONF;

pub fn run() -> Rocket<Build> {
    let config = rocket::Config::figment()
        .merge(("port", APP_CONF.server.inet.port()))
        .merge(("address", APP_CONF.server.inet.ip()))
        .merge(("workers", APP_CONF.server.workers))
        .merge(("secret_key", APP_CONF.server.secret_key.as_str()))
        .merge((
            "template_dir",
            APP_CONF.assets.path.join("./templates").to_str().unwrap(),
        ));

    // Build and run Rocket instance
    rocket::build()
        .configure(config)
        .attach(Template::fairing())
        .mount(
            "/",
            routes![
                routes::get_index,
                routes::get_robots,
                routes::get_initiate_base,
                routes::get_initiate_login,
                routes::get_initiate_signup,
                routes::get_initiate_recover,
                routes::get_initiate_logout,
                routes::get_dashboard_base,
                routes::get_dashboard_welcome,
                routes::get_dashboard_trackers,
                routes::get_dashboard_payouts,
                routes::get_dashboard_payouts_partial_payouts,
                routes::get_dashboard_account,
                routes::get_assets_fonts,
                routes::get_assets_images,
                routes::get_assets_stylesheets,
                routes::get_assets_javascripts,
                routes::post_initiate_login_form_login,
                routes::post_initiate_signup_form_signup,
                routes::post_initiate_recover_form_recover,
                routes::post_dashboard_trackers_form_create,
                routes::post_dashboard_trackers_form_remove,
                routes::post_dashboard_payouts_form_request,
                routes::post_dashboard_account_form_account,
                routes::post_dashboard_account_form_payout,
                routes::post_track_payment,
                routes::post_track_signup,
                routes::post_management_account,
            ],
        )
        .register("/", catchers![catchers::forbidden, catchers::gone,])
        .manage(db::pool())
}
