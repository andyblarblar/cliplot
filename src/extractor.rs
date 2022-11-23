//! Stdin data extractor runtime

use crate::Message;
use chrono::{DateTime, Utc};
use iced::Subscription;
use regex::Regex;
use std::sync::Arc;
use tokio::io::{stdin, AsyncReadExt, Stdin};

/// Data parsed from the CLI
#[derive(Copy, Clone, Default, Debug)]
pub struct Data {
    /// The time the data was read from the CLI
    pub stamp: DateTime<Utc>,
    /// Channel the data is from. Starts at 0
    pub channel: usize,
    /// Reading parsed from regex
    pub data: f64,
}

/// Extractor configuration. This configures the regex for each channel, and implicitly defines the
/// number of channels via the number of regex matchers.
///
/// # Assumptions
/// - Regex matchers can do anything, but must have a single capture group who's matching text must be convertable to a float.
pub struct Config {
    /// Regex expression for each channel, ordered by index
    pub matchers: Vec<Regex>,
    //TODO add support for writing to CSV
}

impl Default for Config {
    /// Crates a config with a single channel, where that single channel just matches literally every
    /// float.
    fn default() -> Self {
        let matchers = vec![Regex::new(r"(\d*)").unwrap()];
        Config { matchers }
    }
}

/// State machine for the extraction stream
enum State {
    Starting(Arc<Config>),
    Working(Stdin, Arc<Config>),
}

/// Subscription that extracts data from stdin using the configured Regex matchers.
pub fn extract_channels(config: Arc<Config>) -> Subscription<Message> {
    struct Pipe;

    iced::subscription::unfold(
        std::any::TypeId::of::<Pipe>(),
        State::Starting(config),
        |state| async {
            match state {
                State::Starting(arc_config) => {
                    let stin = stdin();

                    (None, State::Working(stin, arc_config))
                }
                State::Working(mut stin, config) => {
                    // Read chunks from stdin
                    let mut buff = [0u8; 11000]; //TODO store the buffer in front of the furthest match, to avoid cutting off data. Linked list?
                    stin.read_exact(&mut buff).await.unwrap();

                    let done_time = Utc::now();

                    let str = String::from_utf8_lossy(&buff);

                    // Batch all readings from each chunk into one message
                    let mut message = Vec::new();

                    // Match on each channel
                    for (i, matcher) in config.matchers.iter().enumerate() {
                        if let Some(captures) = matcher.captures(&str) {
                            log::trace!("Capture was: {}", &captures[1]);

                            // Assume one capture group on each regex, with only a floating point number in it
                            if let Ok(data) = captures[1].parse::<f64>() {
                                log::trace!("Read: {} on chan {}", data, i);

                                message.push(Data {
                                    stamp: done_time,
                                    channel: i,
                                    data,
                                });
                            }
                        }
                    }

                    (Some(Message::Data(message)), State::Working(stin, config))
                }
            }
        },
    )
}
