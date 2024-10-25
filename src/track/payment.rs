// Raider
//
// Affiliates dashboard
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use bigdecimal::BigDecimal;
use chrono::offset::Utc;
use diesel;
use diesel::prelude::*;
use log;
use num_traits::ToPrimitive;
use separator::FixedPlaceSeparatable;
use std::thread;

use crate::exchange::manager::normalize as exchange_normalize;
use crate::notifier::email::EmailNotifier;
use crate::storage::db::DbConn;
use crate::storage::models::{Account, Tracker};
use crate::storage::schemas::account::dsl::account;
use crate::storage::schemas::balance::dsl::{
    account_id as balance_account_id, amount as balance_amount, balance,
    created_at as balance_created_at, currency as balance_currency, trace as balance_trace,
    tracker_id as balance_tracker_id, updated_at as balance_updated_at,
};
use crate::storage::schemas::tracker::dsl::{
    id as tracker_id, statistics_signups as tracker_statistics_signups, tracker,
    updated_at as tracker_updated_at,
};
use crate::APP_CONF;

pub enum HandlePaymentError {
    InvalidAmount,
    BadCurrency,
    NotFound,
}

pub enum HandleSignupError {
    NotFound,
}

pub fn handle_payment(
    db: &mut DbConn,
    tracking_id: &str,
    amount_real: f32,
    currency: &str,
    trace: &Option<String>,
) -> Result<Option<(bool, String, String, f32, String)>, HandlePaymentError> {
    log::debug!(
        "payment track handle: {} of real amount: {} {}",
        tracking_id,
        currency,
        amount_real
    );

    // Normalize amount
    if let Ok(amount) = exchange_normalize(amount_real, currency) {
        log::debug!(
            "normalized real amount: {} {} to: {} {}",
            currency,
            amount_real,
            &APP_CONF.payout.currency,
            amount
        );

        // Validate amount
        if amount < 0.00 {
            return Err(HandlePaymentError::InvalidAmount);
        }

        // Ignore zero amount
        if amount == 0.00 {
            return Ok(None);
        }

        // Resolve user for tracking code
        let track_result = tracker
            .filter(tracker_id.eq(tracking_id))
            .inner_join(account)
            .first::<(Tracker, Account)>(&mut **db);

        if let Ok(track_inner) = track_result {
            // Apply user commission percentage to amount
            let commission_amount = amount * track_inner.1.commission.to_f32().unwrap_or(0.0);

            if commission_amount > 0.0 {
                let now_date = Utc::now().naive_utc();

                let insert_result = diesel::insert_into(balance)
                    .values((
                        &balance_amount.eq(commission_amount as f64),
                        &balance_currency.eq(&APP_CONF.payout.currency),
                        &balance_trace.eq(trace),
                        &balance_account_id.eq(&track_inner.1.id),
                        &balance_tracker_id.eq(&track_inner.0.id),
                        &balance_created_at.eq(&now_date),
                        &balance_updated_at.eq(&now_date),
                    ))
                    .execute(&mut **db);

                if insert_result.is_ok() == true {
                    return Ok(Some((
                        track_inner.1.notify_balance,
                        track_inner.1.email.to_owned(),
                        track_inner.0.id.to_owned(),
                        commission_amount,
                        APP_CONF.payout.currency.to_owned(),
                    )));
                }
            }
        }

        log::warn!(
            "payment track: {} could not be stored to balance for amount: {} {}",
            tracking_id,
            currency,
            amount
        );

        Err(HandlePaymentError::NotFound)
    } else {
        Err(HandlePaymentError::BadCurrency)
    }
}

pub fn handle_signup(db: &mut DbConn, tracking_id: &str) -> Result<(), HandleSignupError> {
    log::debug!("signup track handle: {}", tracking_id);

    // Resolve tracking code
    let tracker_result = tracker
        .filter(tracker_id.eq(tracking_id))
        .first::<Tracker>(&mut **db);

    if let Ok(tracker_inner) = tracker_result {
        // Notice: this increment is not atomic; thus it is not 100% safe. We do this for \
        //   simplicity as Diesel doesnt seem to provide a way to do an increment in the query.
        let update_result = diesel::update(tracker.filter(tracker_id.eq(tracking_id)))
            .set((
                tracker_statistics_signups.eq(tracker_inner.statistics_signups + 1),
                tracker_updated_at.eq(Utc::now().naive_utc()),
            ))
            .execute(&mut **db);

        if update_result.is_ok() == true {
            return Ok(());
        }
    }

    log::warn!("signup track: {} could not be stored", tracking_id);

    Err(HandleSignupError::NotFound)
}

pub fn run_notify_payment(
    user_email: String,
    source_tracker_id: String,
    commission_amount: f32,
    commission_currency: String,
) {
    thread::spawn(move || {
        dispatch_notify_payment(
            user_email,
            source_tracker_id,
            commission_amount,
            commission_currency,
        );
    });
}

fn dispatch_notify_payment(
    user_email: String,
    source_tracker_id: String,
    commission_amount: f32,
    commission_currency: String,
) {
    // Generate message
    let mut message = String::new();

    message.push_str("Hi,\n\n");

    message.push_str(&format!(
        "You just received commission money of {} {} on your affiliates account balance.\n",
        &commission_currency,
        &commission_amount.separated_string_with_fixed_place(2)
    ));

    message.push_str(&format!(
        "This commission was generated by your tracker with ID: {}\n\n",
        &source_tracker_id
    ));

    message.push_str(
        "You can request for a payout anytime on your dashboard (payouts are not automatic).",
    );

    // Send email
    if EmailNotifier::dispatch(
        &user_email,
        "You received commission money".to_string(),
        &message,
    )
    .is_ok()
        == true
    {
        log::debug!(
            "sent balance commission notification email to user on: {}",
            user_email
        );
    } else {
        log::error!(
            "could not send balance commission notification email to user on: {}",
            user_email
        );
    }
}
