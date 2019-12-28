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
mod tests {
    use super::*;

    use crate::line_;

    #[macro_export]
    macro_rules! agency {
        ($name:literal, [$($line:ident),*]) => (
            Agency::new($name.to_string(), vec![$($crate::line_!($line)),*])
        );
        (pubtrans, [$($line:ident),*]) => (
            agency!("Public Transport", [$($line),*])
        );
    }

    #[test]
    fn test_getters() {
        let agency = agency!(pubtrans, [blue]);
        assert_eq!(agency.name(), "Public Transport");
        assert_eq!(agency.lines(), &[line_!(blue)]);
    }
}
