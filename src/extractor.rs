//! Stdin data extractor runtime

use crate::extractor::State::Closed;
use crate::Message;
use chrono::{DateTime, Utc};
use csv::Writer;
use iced::Subscription;
use regex::Regex;
use std::fs::File;
use std::path::PathBuf;
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
/// Its suggested to use delimiters, else the number can get split over a buffer divide.
pub struct Config {
    /// Regex expression for each channel, ordered by index
    pub matchers: Vec<Regex>,
    /// CSV file to write to, if set
    pub csv: Option<PathBuf>,
}

impl Default for Config {
    /// Crates a config with a single channel, where that single channel just matches a float deliminated
    /// by $.
    fn default() -> Self {
        let matchers = vec![Regex::new(r"\$([+|-]?\d*\.?\d*)\$").unwrap()];
        Self {
            matchers,
            csv: None,
        }
    }
}

/// State machine for the extraction stream
enum State {
    Starting(Arc<Config>),
    Working(Stdin, Arc<Config>, String, Box<Option<Writer<File>>>, i64),
    Closed,
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

                    // Create CSV and write headers if configured
                    let csv = arc_config.csv.clone().map(|path| {
                        let mut csv = csv::WriterBuilder::new()
                            .from_path(&path)
                            .expect("CSV path error!");

                        csv.write_record(
                            &["Timestamp".to_string(), "Data".to_string(), "Channel".to_string()]
                        )
                        .expect("CSV write error!");

                        csv
                    });

                    (
                        None,
                        State::Working(
                            stin,
                            arc_config,
                            String::new(),
                            Box::new(csv),
                            Utc::now().timestamp_millis(),
                        ),
                    )
                }
                State::Working(mut stin, config, mut working_str, mut csv, start_time) => {
                    // Read chunks from stdin, this needs to be small else it is actually much slower due to regex scaling with input size
                    let mut buff = [0u8; 4];
                    if stin.read_exact(&mut buff).await.is_err() {
                        log::error!("Stdin Closed");

                        // Write all data out to the csv before we die
                        if let Some(mut csv) = *csv {
                            log::debug!("Flushing csv writer");
                            let _ = csv.flush();
                            drop(csv);
                        }

                        // Signal stdin was closed to stop from freezing gui
                        return (Some(Message::Closed), Closed);
                    }

                    let done_time = Utc::now();

                    // Extend working string
                    let str = String::from_utf8_lossy(&buff);
                    working_str.push_str(&str);

                    // Batch all readings from each chunk into one message
                    let mut message = Vec::new();

                    let mut furthest_capture = -1isize;

                    // Match on each channel
                    for (i, matcher) in config.matchers.iter().enumerate() {
                        let mut captures = matcher.capture_locations();

                        for matches in matcher.find_iter(&working_str) {
                            // Keep track of the furthest offset to shrink working string
                            if matches.end() as isize > furthest_capture {
                                furthest_capture = matches.end() as isize;
                            }

                            //Read capture at the found spot to avoid polynomial time search
                            matcher.captures_read_at(&mut captures, &working_str, matches.start());
                            let bounds = captures.get(1).unwrap();

                            // Assume one capture group on each regex, with only a floating point number in it
                            if let Ok(data) = &working_str[bounds.0..bounds.1].parse::<f64>() {
                                log::trace!("data: {}", *data);
                                message.push(Data {
                                    stamp: done_time,
                                    channel: i,
                                    data: *data,
                                });
                            }
                        }
                    }

                    // Remove working string data from before the furthest match, since it's impossible for us to miss
                    // a match since its already been checked. It's still possible for part of a match to be
                    // present after this last match, so keep it around until we match again.
                    if furthest_capture != -1 {
                        working_str.drain(0..furthest_capture as usize);
                    }

                    // Write to CSV if configured
                    if let Some(ref mut csv) = *csv {
                        message.iter().for_each(|m| {
                            csv.write_record(&[
                                format!("{}", m.stamp.timestamp_millis() - start_time),
                                format!("{}", m.data),
                                format!("{}", m.channel),
                            ])
                            .expect("Failed to write to CSV!");
                        });
                    }

                    (
                        Some(Message::Data(message)),
                        State::Working(stin, config, working_str, csv, start_time),
                    )
                }
                Closed => (Some(Message::Closed), Closed),
            }
        },
    )
}
