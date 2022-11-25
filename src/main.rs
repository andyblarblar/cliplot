mod cli;
mod extractor;
mod interface;

use crate::extractor::Config;
use crate::interface::*;
use clap::Parser;
use iced::{Application, Settings};
use regex::Regex;
use simplelog::ConfigBuilder;
use std::sync::Arc;

fn main() {
    let args = cli::Args::parse();

    simplelog::SimpleLogger::init(
        args.verbose.log_level_filter(),
        ConfigBuilder::new()
            .add_filter_allow("cliplot".to_string())
            .build(),
    )
    .unwrap();

    log::debug!("Regex Vec: {:?}", args.regexes);
    log::debug!("csv path: {:?}", args.csv);

    log::info!("Creating gui...");

    State::run(Settings {
        antialiasing: true,
        default_font: Some(include_bytes!("../fonts/notosans-regular.ttf")),
        flags: Flags {
            extractor_conf: Arc::new(Config {
                matchers: args.regexes.map_or_else(
                    || Config::default().matchers,
                    |r| {
                        r.iter()
                            .map(|s| Regex::new(s).expect("Invalid Regex!"))
                            .collect()
                    },
                ),
                csv: args.csv,
            }),
        },
        ..Settings::default()
    })
    .unwrap();
}
