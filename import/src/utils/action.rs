#[cfg(feature = "progress")]
use std::time::Instant;

#[cfg(feature = "progress")]
use console::Term;
#[cfg(feature = "progress")]
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use serde::de::DeserializeOwned;

use super::Dataset;

#[cfg(feature = "progress")]
pub(crate) struct Action {
    message: &'static str,
    started: Instant,
    needs_last_line_cleared: bool,
}

#[cfg(feature = "progress")]
impl Action {
    pub(crate) fn start(message: &'static str) -> Self {
        Self {
            message,
            started: Instant::now(),
            needs_last_line_cleared: false,
        }
    }

    fn percent_bar(size: u64, message: &str) -> ProgressBar {
        let style =
            ProgressStyle::default_bar().template("{msg} {wide_bar} {percent:>3}% {elapsed:>4}");
        let progress_bar = ProgressBar::new(size).with_style(style);
        progress_bar.set_message(message);
        progress_bar
    }

    pub(crate) fn read_csv<'s, D, S>(
        &self,
        dataset: &'s mut S,
        name: &str,
    ) -> Result<impl Iterator<Item = csv::Result<D>> + 's, S::Error>
    where
        D: DeserializeOwned + 's,
        S: Dataset,
    {
        let table = dataset.open_csv(name)?;
        let reader = Self::percent_bar(table.size, self.message).wrap_read(table.reader);
        Ok(csv::Reader::from_reader(reader).into_deserialize())
    }

    pub(crate) fn wrap_iter<T>(&mut self, collection: T) -> impl Iterator<Item = T::Item>
    where
        T: IntoIterator,
    {
        self.needs_last_line_cleared = true;
        let iterator = collection.into_iter();
        let progress_bar = Self::percent_bar(iterator.size_hint().0 as u64, self.message);
        iterator.progress_with(progress_bar)
    }

    pub(crate) fn complete(self, message: &str) {
        let elapsed = self.started.elapsed().as_millis() as f64 / 1000.0;
        let term = Term::stderr();
        let width = term.size().1 as usize - message.len() - 3;
        if self.needs_last_line_cleared {
            term.clear_last_lines(1).unwrap();
        }
        eprintln!("{} {:>width$.2}s", message, elapsed, width = width);
    }
}

#[cfg(not(feature = "progress"))]
pub(crate) struct Action;

#[cfg(not(feature = "progress"))]
impl Action {
    pub(crate) fn start(_message: &'static str) -> Self {
        Self
    }

    pub(crate) fn read_csv<'s, D, S>(
        &self,
        dataset: &'s mut S,
        name: &str,
    ) -> Result<impl Iterator<Item = csv::Result<D>> + 's, S::Error>
    where
        D: DeserializeOwned + 's,
        S: Dataset,
    {
        let table = dataset.open_csv(name)?;
        Ok(csv::Reader::from_reader(table.reader).into_deserialize())
    }

    pub(crate) fn wrap_iter<T>(&mut self, collection: T) -> impl Iterator<Item = T::Item>
    where
        T: IntoIterator,
    {
        collection.into_iter()
    }

    pub(crate) fn complete(self, _message: &str) {}
}
