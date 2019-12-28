use chrono::NaiveDate;

use simulation::Color;
use crate::create_id_type;
use crate::trip::Route;
use super::Kind;

create_id_type!(LineId);

#[derive(Debug, PartialEq)]
pub(crate) struct Line {
    name: String,
    color: Color,
    pub(crate) kind: Kind,
    pub(crate) routes: Vec<Route>,
}

impl Line {
    pub(crate) fn new(name: String, color: Color, kind: Kind, routes: Vec<Route>) -> Line {
        Line {
            name,
            color,
            kind,
            routes,
        }
    }

    pub(crate) fn freeze(&self, date: NaiveDate) -> (Color, serialization::Line) {
        let route = self.routes.iter()
            .max_by_key(|route| route.num_trips_at(date))
            .unwrap();
        let nodes = route.freeze_nodes();
        let trains = route.freeze_trains(date);
        let color = self.color.clone();
        (color, serialization::Line::new(self.name.clone(), nodes, trains))
    }
}

#[cfg(test)]
mod tests {
    #[macro_export]
    macro_rules! line_ {
        ($name:expr, $color:expr, $kind:ident) => (
            $crate::line::Line::new(
                $name.to_string(),
                simulation::Color::new($color.0, $color.1, $color.2),
                $crate::line::Kind::$kind,
                Vec::new()
            )
        );
        (blue) => (
            $crate::line_!("Blue Line", (0, 0, 255), SuburbanRailway)
        );
        (blue-replacement) => (
            $crate::line_!("Blue Line", (0, 0, 255), Bus)
        );
        (green) => (
            $crate::line_!("Green Line", (0, 255, 0), SuburbanRailway)
        );
    }
}
