#![allow(clippy::module_inception, clippy::zero_prefixed_literal)]

use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;

use zip::ZipArchive;

mod agency;
pub mod coord;
mod deserialize;
pub mod line;
mod location;
pub mod profile;
mod service;
pub mod shape;
pub mod trip;
mod utils;

#[cfg(test)]
mod fixtures;

use crate::agency::Agency;
use crate::line::Line;
use crate::location::Linearizer;
use crate::profile::Profile;
use crate::shape::SmoothMode;
use crate::trip::Scheduler;
use crate::utils::Dataset;

pub struct ImportedDataset {
    agencies: Vec<Agency>,
}

impl ImportedDataset {
    fn fetch(
        mut dataset: impl Dataset,
        shape_smoothing: SmoothMode,
    ) -> Result<Self, Box<dyn Error>> {
        let services = service::Importer::import(&mut dataset)?;
        let locations = location::Importer::import(&mut dataset)?;
        let shapes = shape::Importer::import(&mut dataset, shape_smoothing)?;
        let line_importer = line::Importer::import(&mut dataset)?;
        let trip_importer = trip::Importer::new(
            &services,
            &locations,
            &shapes,
            line_importer.id_mapping(),
            line_importer.line_count(),
        );
        let routes = trip_importer.import(&mut dataset)?;
        let lines = line_importer.finish(routes)?;
        let agencies = agency::Importer::import(&mut dataset, lines)?;
        Ok(Self { agencies })
    }

    pub fn import(
        path: impl AsRef<OsStr>,
        shape_smoothing: SmoothMode,
    ) -> Result<Self, Box<dyn Error>> {
        let path = Path::new(&path);
        if path.is_dir() {
            let mut path = PathBuf::from(&path);
            path.push(".txt");
            Self::fetch(path, shape_smoothing)
        } else {
            let archive = ZipArchive::new(File::open(&path)?)?;
            Self::fetch(archive, shape_smoothing)
        }
    }

    pub fn agencies(&self) -> impl Iterator<Item = &Agency> {
        self.agencies.iter()
    }

    fn store(&self, profile: Profile, date: NaiveDate) -> storage::Dataset {
        let mut linearizer = Linearizer::new();
        let mut scheduler = Scheduler::new();
        let lines = profile
            .filter(self.agencies())
            .into_iter()
            .map(|line| line.store(date, &mut linearizer, &mut scheduler))
            .collect();

        let stations = linearizer
            .into_iter()
            .map(|location| location.store())
            .collect();

        storage::Dataset::new(stations, scheduler.schedules(), lines)
    }

    pub fn store_into(
        &self,
        writer: impl Write,
        profile: Profile,
        date: NaiveDate,
    ) -> bincode::Result<()> {
        let dataset = self.store(profile, date);
        bincode::serialize_into(writer, &dataset)
    }
}
