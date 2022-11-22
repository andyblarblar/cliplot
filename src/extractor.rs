//! Stdin data extractor runtime

use chrono::{DateTime, Utc};

#[derive(Copy, Clone, Default, Debug)]
pub struct Data {
    pub stamp: DateTime<Utc>,
    pub channel: usize,
    pub data: f64,
}

//TODO make function that returns a subscription that produces data from stdin