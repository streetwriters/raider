// Raider
//
// Affiliates dashboard
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use separator::FixedPlaceSeparatable;
use bigdecimal::BigDecimal;
use num_traits::cast::ToPrimitive;
use diesel::prelude::*;
use diesel::dsl::sum;

use storage::schemas::balance::dsl::{balance, account_id as balance_account_id,
                                     amount as balance_amount, released as balance_released};
use storage::db::DbConn;

pub fn get_balance(db: &DbConn, user_id: i32, released: Option<bool>) -> f32 {
    let balance_result = if let Some(released_inner) = released {
        balance
            .filter(balance_account_id.eq(user_id))
            .filter(balance_released.eq(released_inner))
            .select(sum(balance_amount))
            .first(&**db)
    } else {
        balance
            .filter(balance_account_id.eq(user_id))
            .select(sum(balance_amount))
            .first(&**db)
    };

    let balance_count: Option<f32> = balance_result.ok().and_then(|value: Option<BigDecimal>| {
        if let Some(value_inner) = value {
            value_inner.to_f32()
        } else {
            None
        }
    });

    balance_count.unwrap_or(0.0)
}

pub fn get_balance_string(db: &DbConn, user_id: i32, released: Option<bool>) -> String {
    get_balance(db, user_id, released).separated_string_with_fixed_place(2)
}

pub fn check_argument_value(argument: &Option<String>, against: &str) -> bool {
    if let &Some(ref value) = argument {
        value == against
    } else {
        false
    }
}
