//! Command line arguments

use clap::Parser;
use clap_verbosity_flag::Verbosity;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Regex strings to parse each channel with. If this is not specified, then a single channel
    /// that parses for `$float$` will be used.
    ///
    /// Each regex should be unambiguous from the others, and contain one capture group that contains
    /// a string convertable to a f64. Deliminators (such as the `$` above) are necessary to avoid
    /// numbers being cut across buffer breaks.
    #[arg(short, long)]
    pub regexes: Option<Vec<String>>,

    #[command(flatten)]
    pub verbose: Verbosity,
}
