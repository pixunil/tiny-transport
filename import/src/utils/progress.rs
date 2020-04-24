use std::time::Instant;

use indicatif::{ProgressBar, ProgressStyle};

pub(crate) fn elapsed(started: Instant) -> f64 {
    started.elapsed().as_millis() as f64 / 1000.0
}

pub(crate) fn percent_bar(size: u64, message: &str) -> ProgressBar {
    let style = ProgressStyle::default_bar().template("{msg} {wide_bar} {percent:>3}%");
    let progress_bar = ProgressBar::new(size).with_style(style);
    progress_bar.set_message(message);
    progress_bar
}
