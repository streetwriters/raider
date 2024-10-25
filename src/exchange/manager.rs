// Raider
//
// Affiliates dashboard
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use log;
use reqwest::blocking::Client;
use reqwest::StatusCode;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

use crate::config::config::ConfigExchangeCurrencyAPI;
use crate::config::config::ConfigExchangeFixer;
use crate::APP_CONF;

const POLL_RATE_SECONDS: u64 = 259200;
const RETRY_POLL_SECONDS: u64 = 60;
const RETRY_POLL_ATTEMPTS_LIMIT: u16 = 2;

lazy_static! {
    static ref RATES: Arc<RwLock<HashMap<String, f32>>> = Arc::new(RwLock::new(HashMap::new()));
    static ref HTTP_CLIENT: Client = Client::builder()
        .timeout(Duration::from_secs(20))
        .gzip(true)
        .build()
        .unwrap();
}

#[derive(Deserialize)]
struct FixerLatestResponse {
    rates: HashMap<String, f32>,
}

fn store_rates(rates: HashMap<String, f32>) {
    let mut store = RATES.write().unwrap();

    *store = rates;
}

fn get_rates_from_fixer(fixer: &ConfigExchangeFixer) -> Result<HashMap<String, f32>, ()> {
    log::debug!("acquiring exchange rates from Fixer");

    let response = HTTP_CLIENT
        .get(&format!(
            "{}/latest?base={}",
            &fixer.endpoint, &APP_CONF.payout.currency
        ))
        .header("apikey", &fixer.api_key)
        .send();

    if let Ok(response_inner) = response {
        let status = response_inner.status();

        log::debug!("received updated exchange rates");

        if status == StatusCode::OK {
            if let Ok(response_json) = response_inner.json::<FixerLatestResponse>() {
                return Ok(response_json.rates);
            }
        }
    }
    return Err(());
}

fn get_rates_from_currency_api(
    api: &ConfigExchangeCurrencyAPI,
) -> Result<HashMap<String, f32>, ()> {
    log::debug!("acquiring exchange rates from Fixer");

    let base_currency = &APP_CONF.payout.currency.to_lowercase();
    let response = HTTP_CLIENT
        .get(&format!(
            "{}/currencies/{}.min.json",
            &api.endpoint, base_currency
        ))
        .send();

    if let Ok(response_inner) = response {
        let status = response_inner.status();

        log::debug!("status {:?} endpint {:?}", status, api.endpoint);
        if status == StatusCode::OK {
            log::debug!("received updated exchange rates");

            if let Ok(json) =
                response_inner.json::<serde_json::map::Map<String, serde_json::Value>>()
            {
                let value = json.get(base_currency);
                if let Some(value) = value {
                    return value.as_object().map_or(Ok(HashMap::default()), |o| {
                        let mut rates = HashMap::new();
                        for (k, v) in o.iter() {
                            if let Some(v) = v.as_f64() {
                                rates.insert(k.to_string(), v as f32);
                            }
                        }
                        Ok(rates)
                    });
                }
            }
        }
    }
    return Err(());
}

fn update_rates(retry_count: u16) -> Result<(), ()> {
    log::debug!("acquiring updated exchange rates");

    let rates = if let Some(fixer) = &APP_CONF.exchange.fixer {
        get_rates_from_fixer(fixer)
    } else if let Some(api) = &APP_CONF.exchange.currency_api {
        get_rates_from_currency_api(api)
    } else {
        Err(())
    };

    if let Ok(rates) = rates {
        log::debug!("got updated exchange rates: {:?}", &rates);

        store_rates(rates);

        return Ok(());
    }

    log::error!("could not request updated exchange rates");
    // Re-schedule an update after a few seconds? (if retry count not over limit)
    if retry_count <= RETRY_POLL_ATTEMPTS_LIMIT {
        log::info!(
            "scheduled an exchange rates update retry in {} seconds",
            RETRY_POLL_SECONDS
        );

        thread::sleep(Duration::from_secs(RETRY_POLL_SECONDS));

        return update_rates(retry_count + 1);
    }

    log::error!(
        "exceeded exchange rates update retry limit of {} attempts",
        RETRY_POLL_ATTEMPTS_LIMIT
    );

    // Failed to update rates (all retry attempts exceeded)
    return Err(());
}

pub fn normalize(amount: f32, currency: &str) -> Result<f32, ()> {
    if currency == APP_CONF.payout.currency {
        Ok(amount)
    } else {
        if let Ok(ref store) = RATES.read() {
            if let Some(rate) = store.get(currency) {
                if rate > &0.0 {
                    Ok((1.0 / rate) * amount)
                } else {
                    Err(())
                }
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}

pub fn run() {
    loop {
        log::debug!("running an exchange poll operation...");

        update_rates(0).ok();

        log::info!("ran exchange poll operation");

        // Hold for next poll run
        thread::sleep(Duration::from_secs(POLL_RATE_SECONDS));
    }
}
