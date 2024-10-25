// Raider
//
// Affiliates dashboard
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use log;
use std::fs::File;
use std::io::Read;
use toml;

use super::config::*;
use crate::APP_ARGS;

pub struct ConfigReader;

impl ConfigReader {
    pub fn make() -> Config {
        log::debug!("reading config file: {}", &APP_ARGS.config);

        let mut file = File::open(&APP_ARGS.config).expect("cannot find config file");
        let mut conf = String::new();

        file.read_to_string(&mut conf)
            .expect("cannot read config file");

        log::debug!("read config file: {}", &APP_ARGS.config);

        toml::from_str(&conf).expect("syntax error in config file")
    }
}
