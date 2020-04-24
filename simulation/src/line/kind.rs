use serde_derive::{Deserialize, Serialize};

use na::Vector2;

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
    pub(crate) fn line_width(self) -> f32 {
        match self {
            Kind::Railway => 50.0,
            Kind::SuburbanRailway | Kind::UrbanRailway => 40.0,
            Kind::WaterTransport => 30.0,
            Kind::Bus | Kind::Tram => 15.0,
        }
    }

    pub fn train_size(self) -> Vector2<f32> {
        match self {
            Kind::Railway | Kind::SuburbanRailway | Kind::UrbanRailway => {
                Vector2::new(220.0, 150.0)
            }
            Kind::WaterTransport => Vector2::new(180.0, 120.0),
            Kind::Tram => Vector2::new(160.0, 100.0),
            Kind::Bus => Vector2::new(130.0, 100.0),
        }
    }

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
