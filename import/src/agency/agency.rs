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

#[cfg(test)]
pub(crate) mod fixtures {
    use super::*;

    pub(crate) fn pubtrans(lines: Vec<Line>) -> Agency {
        Agency {
            name: "Public Transport".to_string(),
            lines,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::agency::fixtures::*;

    #[test]
    fn test_getters() {
        let agency = agencies::pubtrans(vec![lines::u4()]);
        assert_eq!(agency.name(), "Public Transport");
        assert_eq!(agency.lines(), &[lines::u4()]);
    }
}
