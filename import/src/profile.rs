use std::convert::TryFrom;
use std::error::Error;
use std::fmt;

use simulation::line::Kind;

use crate::agency::Agency;
use crate::line::Line;
use serde::export::Formatter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    BerlinSuburbanRailway,
    BerlinUrbanRailway,
    BerlinRapidTransit,
    BerlinMetro,
    BerlinWithoutRailway,
    Berlin,
    BerlinBrandenburgWithoutRailway,
    BerlinBrandenburg,
}

impl Profile {
    pub(crate) fn filter<'a>(self, agencies: impl Iterator<Item = &'a Agency>) -> Vec<&'a Line> {
        let matching_agencies = agencies
            .filter(|agency| self.matches_agency(agency))
            .collect::<Vec<_>>();

        matching_agencies
            .into_iter()
            .flat_map(|agency| agency.lines())
            .filter(|line| self.matches_line(line))
            .collect()
    }

    fn matches_agency(self, agency: &Agency) -> bool {
        match self {
            Self::BerlinSuburbanRailway
            | Self::BerlinUrbanRailway
            | Self::BerlinRapidTransit
            | Self::BerlinMetro
            | Self::BerlinWithoutRailway
            | Self::Berlin => {
                ["Berliner Verkehrsbetriebe", "S-Bahn Berlin GmbH"].contains(&agency.name())
            }
            Self::BerlinBrandenburgWithoutRailway | Self::BerlinBrandenburg => true,
        }
    }

    fn matches_line(self, line: &Line) -> bool {
        let is_rapid = [Kind::SuburbanRailway, Kind::UrbanRailway].contains(&line.kind());
        match self {
            Self::BerlinSuburbanRailway => line.kind() == Kind::SuburbanRailway,
            Self::BerlinUrbanRailway => line.kind() == Kind::UrbanRailway,
            Self::BerlinRapidTransit => is_rapid,
            Self::BerlinMetro => is_rapid || line.name().starts_with('M'),
            Self::BerlinWithoutRailway | Self::BerlinBrandenburgWithoutRailway => {
                line.kind() != Kind::Railway
            }
            Self::Berlin | Self::BerlinBrandenburg => true,
        }
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self::BerlinWithoutRailway
    }
}

#[derive(Debug, Clone)]
pub struct InvalidProfileError(String);

impl fmt::Display for InvalidProfileError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "profile '{}' not found", self.0)
    }
}

impl Error for InvalidProfileError {}

impl TryFrom<&str> for Profile {
    type Error = InvalidProfileError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "berlin-s" => Ok(Self::BerlinSuburbanRailway),
            "berlin-u" => Ok(Self::BerlinUrbanRailway),
            "berlin-s+u" => Ok(Self::BerlinRapidTransit),
            "berlin-s+u+metro" => Ok(Self::BerlinMetro),
            "berlin-no-r" => Ok(Self::BerlinWithoutRailway),
            "berlin" => Ok(Self::Berlin),
            "berlin-brandenburg-no-r" => Ok(Self::BerlinWithoutRailway),
            "berlin-brandenburg" => Ok(Self::Berlin),
            _ => Err(InvalidProfileError(value.to_string())),
        }
    }
}
