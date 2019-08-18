use std::error::Error;
use std::rc::Rc;
use std::fmt;
use std::collections::{HashSet, HashMap};

use serde::Deserializer;
use serde::de::{Deserialize, Visitor, Error as DeserializeError};

use chrono::prelude::*;

use super::utils::*;

#[derive(Debug, PartialEq, Eq)]
pub struct Service {
    start: NaiveDate,
    end: NaiveDate,
    weekdays: [bool; 7],
    added: HashSet<NaiveDate>,
    removed: HashSet<NaiveDate>,
}

impl Service {
    pub fn new(start: NaiveDate, end: NaiveDate, weekdays: [bool; 7]) -> Service {
        Service {
            start,
            end,
            weekdays,
            added: HashSet::new(),
            removed: HashSet::new(),
        }
    }

    fn add_exception(&mut self, record: ServiceExceptionRecord) {
        let exception_set = match record.exception_type {
            ServiceExceptionType::Added => &mut self.added,
            ServiceExceptionType::Removed => &mut self.removed,
        };
        exception_set.insert(record.date);
    }

    pub fn available_at(&self, date: NaiveDate) -> bool {
        self.added.contains(&date) || (!self.removed.contains(&date) && self.regulary_available_at(date))
    }

    fn regulary_available_at(&self, date: NaiveDate) -> bool {
        let day = date.weekday().num_days_from_monday() as usize;
        self.start <= date && date <= self.end && self.weekdays[day]
    }
}

impl From<ServiceRecord> for Service {
    fn from(record: ServiceRecord) -> Service {
        let weekdays = [record.monday, record.tuesday, record.wednesday, record.thursday, record.friday, record.saturday, record.sunday];
        Service::new(record.start_date, record.end_date, weekdays)
    }
}

pub struct Importer;

impl Importer {
    pub fn import(dataset: &mut impl Dataset) -> Result<HashMap<Id, Rc<Service>>, Box<dyn Error>> {
        let mut services = Self::import_services(dataset)?;
        Self::add_service_exceptions(dataset, &mut services)?;

        let services = services.into_iter()
            .map(|(id, service)| (id, Rc::new(service)))
            .collect();

        Ok(services)
    }

    fn import_services(dataset: &mut impl Dataset) -> Result<HashMap<Id, Service>, Box<dyn Error>> {
        let mut services = HashMap::new();
        let mut reader = dataset.read_csv("calendar.txt")?;
        for result in reader.deserialize() {
            let record: ServiceRecord = result?;
            let id = record.service_id.clone();
            services.insert(id, Service::from(record));
        }
        Ok(services)
    }

    fn add_service_exceptions(dataset: &mut impl Dataset, services: &mut HashMap<Id, Service>) -> Result<(), Box<dyn Error>> {
        let mut reader = dataset.read_csv("calendar_dates.txt")?;
        for result in reader.deserialize() {
            let record: ServiceExceptionRecord = result?;
            services.get_mut(&record.service_id).unwrap().add_exception(record);
        }
        Ok(())
    }
}


#[derive(Debug, Deserialize)]
struct ServiceRecord {
    service_id: Id,
    #[serde(deserialize_with = "deserialize::naive_date")]
    start_date: NaiveDate,
    #[serde(deserialize_with = "deserialize::naive_date")]
    end_date: NaiveDate,
    #[serde(deserialize_with = "deserialize::numeric_bool")]
    monday: bool,
    #[serde(deserialize_with = "deserialize::numeric_bool")]
    tuesday: bool,
    #[serde(deserialize_with = "deserialize::numeric_bool")]
    wednesday: bool,
    #[serde(deserialize_with = "deserialize::numeric_bool")]
    thursday: bool,
    #[serde(deserialize_with = "deserialize::numeric_bool")]
    friday: bool,
    #[serde(deserialize_with = "deserialize::numeric_bool")]
    saturday: bool,
    #[serde(deserialize_with = "deserialize::numeric_bool")]
    sunday: bool,
}

#[derive(Debug, PartialEq, Eq)]
enum ServiceExceptionType {
    Added,
    Removed,
}

impl<'de> Deserialize<'de> for ServiceExceptionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        struct ServiceExceptionTypeVisitor;

        impl<'de> Visitor<'de> for ServiceExceptionTypeVisitor {
            type Value = ServiceExceptionType;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("either 1 or 2")
            }

            fn visit_u64<E>(self, value: u64) -> Result<ServiceExceptionType, E>
                where E: DeserializeError
            {
                match value {
                    1 => Ok(ServiceExceptionType::Added),
                    2 => Ok(ServiceExceptionType::Removed),
                    _ => Err(E::custom(format!("unknown expection type of value: {}", value))),
                }
            }
        }

        deserializer.deserialize_u64(ServiceExceptionTypeVisitor)
    }
}

#[derive(Debug, Deserialize)]
struct ServiceExceptionRecord {
    service_id: Id,
    #[serde(deserialize_with = "deserialize::naive_date")]
    date: NaiveDate,
    exception_type: ServiceExceptionType,
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_test::{Token, assert_de_tokens, assert_de_tokens_error};

    #[macro_export]
    macro_rules! service {
        ($start:expr, $end:expr, $weekdays:expr) => ({
            let start = NaiveDate::from_ymd($start[0], $start[1], $start[2]);
            let end = NaiveDate::from_ymd($end[0], $end[1], $end[2]);
            Service::new(start, end, $weekdays)
        });
        (mon-fri) => (
            $crate::service!([2019, 1, 1], [2019, 12, 31], [true, true, true, true, true, false, false])
        );
    }

    #[test]
    fn test_import_service() {
        let record = ServiceRecord {
            service_id: "1".to_string(),
            start_date: NaiveDate::from_ymd(2019, 1, 1),
            end_date: NaiveDate::from_ymd(2019, 12, 31),
            monday: true,
            tuesday: true,
            wednesday: true,
            thursday: true,
            friday: true,
            saturday: false,
            sunday: false,
        };
        assert_eq!(Service::from(record), service!(mon-fri));
    }

    #[test]
    fn test_add_include_exception_to_service() {
        let mut service = service!(mon-fri);
        let exception = ServiceExceptionRecord {
            service_id: "1".to_string(),
            date: NaiveDate::from_ymd(2019, 1, 5),
            exception_type: ServiceExceptionType::Added,
        };
        service.add_exception(exception);
        assert_eq!(service.added, vec![NaiveDate::from_ymd(2019, 1, 5)].into_iter().collect());
        assert!(service.removed.is_empty());
    }

    #[test]
    fn test_add_exclude_exception_to_service() {
        let mut service = service!(mon-fri);
        let exception = ServiceExceptionRecord {
            service_id: "1".to_string(),
            date: NaiveDate::from_ymd(2019, 12, 24),
            exception_type: ServiceExceptionType::Removed,
        };
        service.add_exception(exception);
        assert_eq!(service.removed, vec![NaiveDate::from_ymd(2019, 12, 24)].into_iter().collect());
        assert!(service.added.is_empty());
    }

    #[test]
    fn test_regulary_available() {
        let service = service!(mon-fri);
        let date = NaiveDate::from_ymd(2019, 1, 7);
        assert!(service.regulary_available_at(date));
        assert!(service.available_at(date));
    }

    #[test]
    fn test_regulary_unavailable() {
        let service = service!(mon-fri);
        let date = NaiveDate::from_ymd(2019, 1, 5);
        assert!(!service.regulary_available_at(date));
        assert!(!service.available_at(date));
    }

    #[test]
    fn test_exceptionally_available() {
        let mut service = service!(mon-fri);
        let date = NaiveDate::from_ymd(2019, 1, 5);
        service.added.insert(date);
        assert!(!service.regulary_available_at(date));
        assert!(service.available_at(date));
    }

    #[test]
    fn test_exceptionally_unavailable() {
        let mut service = service!(mon-fri);
        let date = NaiveDate::from_ymd(2019, 1, 7);
        service.removed.insert(date);
        assert!(service.regulary_available_at(date));
        assert!(!service.available_at(date));
    }

    #[test]
    fn test_deserialize_exception_type() {
        assert_de_tokens(&ServiceExceptionType::Added, &[Token::U8(1)]);
        assert_de_tokens(&ServiceExceptionType::Removed, &[Token::U8(2)]);
        assert_de_tokens_error::<ServiceExceptionType>(&[Token::U8(0)],
            "unknown expection type of value: 0");
        assert_de_tokens_error::<ServiceExceptionType>(&[Token::Str("")],
            "invalid type: string \"\", expected either 1 or 2");
    }

    #[test]
    fn test_from_csv() {
        let mut dataset = crate::dataset!(
            calendar:
                service_id, monday, tuesday, wednesday, thursday, friday, saturday, sunday, start_date, end_date;
                1,          1,      1,       1,         1,        1,      0,        0,      20190101,   20191231
            calendar_dates:
                service_id, date,     exception_type;
                1,          20190105, 1;
                1,          20190107, 2

        );

        let mut service = service!(mon-fri);
        service.added.insert(NaiveDate::from_ymd(2019, 1, 5));
        service.removed.insert(NaiveDate::from_ymd(2019, 1, 7));

        let services = Importer::import(&mut dataset).unwrap();
        assert_eq!(services.len(), 1);
        assert_eq!(*services["1"], service);
    }
}
