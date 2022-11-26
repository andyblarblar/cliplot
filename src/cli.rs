//! Command line arguments

use std::path::{PathBuf};
use clap::Parser;
use clap_verbosity_flag::Verbosity;

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(long_about = "cliplot will live plot data piped into stdin. It can plot multiple channels, parse data with regex, save data to a CSV, and more.")]
pub struct Args {
    /// Regex strings to parse each channel with. If this is not specified, then a single channel
    /// that parses for `$float$` will be used.
    ///
    /// Each regex should be unambiguous from the others, and contain one capture group that contains
    /// a string convertable to a f64. Deliminators (such as the `$` above) are necessary to avoid
    /// numbers being cut across buffer breaks.
    #[arg(short, long)]
    pub regexes: Option<Vec<String>>,
    /// Writes read data into a CSV file at path if set.
    ///
    /// The CSV file will contain the timestamp of each reading in ms, followed by the data and finally the channel number.
    #[arg(long)]
    pub csv: Option<PathBuf>,
    #[command(flatten)]
    pub verbose: Verbosity,
}
