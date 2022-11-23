mod extractor;
mod interface;

use crate::interface::*;
use iced::{Application, Settings};
use log::LevelFilter;
use simplelog::{ConfigBuilder};

fn main() {
    simplelog::SimpleLogger::init(
        LevelFilter::Trace,
        ConfigBuilder::new()
            .add_filter_allow("cliplot".to_string())
            .build(),
    )
    .unwrap();

    State::run(Settings {
        antialiasing: true,
        default_font: Some(include_bytes!("../fonts/notosans-regular.ttf")),
        flags: Flags {
            extractor_conf: Default::default(), //TODO grab from CLI args
        },
        ..Settings::default()
    })
    .unwrap();
}
