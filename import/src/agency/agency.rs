use crate::create_id_type;
use crate::Line;

create_id_type!(AgencyId);

#[derive(Debug, PartialEq)]
pub(crate) struct Agency {
    name: String,
    lines: Vec<Line>,
}

impl Agency {
    pub(crate) fn new(name: String, lines: Vec<Line>) -> Agency {
        Agency { name, lines }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn lines(&self) -> &[Line] {
        &self.lines
    }
}
