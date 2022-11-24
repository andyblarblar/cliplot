mod cli;
mod extractor;
mod interface;

use crate::extractor::Config;
use crate::interface::*;
use clap::Parser;
use iced::{Application, Settings};
use log::LevelFilter;
use regex::Regex;
use simplelog::ConfigBuilder;
use std::sync::Arc;

fn main() {
    simplelog::SimpleLogger::init(
        LevelFilter::Trace,
        ConfigBuilder::new()
            .add_filter_allow("cliplot".to_string())
            .build(),
    )
    .unwrap();

    let args = cli::Args::parse();

    log::trace!("Regex Vec: {:?}", args.regexes);

    State::run(Settings {
        antialiasing: true,
        default_font: Some(include_bytes!("../fonts/notosans-regular.ttf")),
        flags: Flags {
            extractor_conf: Arc::new(Config {
                matchers: args.regexes.map_or_else(|| Config::default().matchers, |r| r.iter()
                        .map(|s| Regex::new(s).expect("Invalid Regex!"))
                        .collect()),
            }),
        },
        ..Settings::default()
    })
    .unwrap();
}
