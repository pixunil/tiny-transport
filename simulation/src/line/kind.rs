use serde_derive::{Serialize, Deserialize};

use crate::color::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Kind {
    Railway,
    SuburbanRailway,
    UrbanRailway,
    Bus,
    Tram,
    WaterTransport,
}

impl Kind {
    pub fn color(self) -> Color {
        match self {
            Kind::Railway => Color::new(227, 0, 27),
            Kind::SuburbanRailway => Color::new(0, 114, 56),
            Kind::UrbanRailway => Color::new(0, 100, 173),
            Kind::Bus => Color::new(125, 23, 107),
            Kind::Tram => Color::new(204, 10, 34),
            Kind::WaterTransport => Color::new(0, 128, 186),
        }
    }
}
