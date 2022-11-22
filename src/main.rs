mod extractor;
mod interface;

use iced::{Application, Settings};
use crate::interface::*;

fn main() {
    State::run(Settings {
        antialiasing: true,
        default_font: Some(include_bytes!("../fonts/notosans-regular.ttf")),
        flags: Flags { num_channels: 1 }, //TODO grab from CLI
        ..Settings::default()
    })
    .unwrap();
}
