use std::error::Error;
use std::rc::Rc;
use std::fmt;
use std::collections::{HashSet, HashMap};
use std::path::PathBuf;

use serde::Deserializer;
use serde::de::{Deserialize, Visitor, Error as DeserializeError};

use chrono::prelude::*;

use super::utils::*;

#[derive(Debug)]
pub struct Service {
    start: NaiveDate,
    end: NaiveDate,
    weekdays: [bool; 7],
    added: HashSet<NaiveDate>,
    removed: HashSet<NaiveDate>,
}

impl Service {
    fn new(record: ServiceRecord) -> (Id, Service) {
        let service = Service {
            start: record.start_date,
            end: record.end_date,
            weekdays: [record.monday, record.tuesday, record.wednesday, record.thursday, record.friday, record.saturday, record.sunday],
            added: HashSet::new(),
            removed: HashSet::new(),
        };
        (record.service_id, service)
    }

    fn add_exception(&mut self, record: ServiceExceptionRecord) {
        let exception_set = match record.exception_type {
            ServiceExceptionType::Added => &mut self.added,
            ServiceExceptionType::Removed => &mut self.removed,
        };
        exception_set.insert(record.date);
    }

    pub fn available_at(&self, date: &NaiveDate) -> bool {
        self.added.contains(date) || (!self.removed.contains(date) && self.regulary_available_at(date))
    }

    fn regulary_available_at(&self, date: &NaiveDate) -> bool {
        let day = date.weekday().num_days_from_monday() as usize;
        &self.start <= date && date <= &self.end && self.weekdays[day]
    }
}

pub fn from_csv(path: &mut PathBuf) -> Result<HashMap<Id, Rc<Service>>, Box<Error>> {
    let mut services = HashMap::new();

    path.set_file_name("calendar.txt");
    let mut reader = csv::Reader::from_path(&path)?;
    for result in reader.deserialize() {
        let (id, service) = Service::new(result?);
        services.insert(id, service);
    }

    path.set_file_name("calendar_dates.txt");
    let mut reader = csv::Reader::from_path(&path)?;
    for result in reader.deserialize() {
        let record: ServiceExceptionRecord = result?;
        services.get_mut(&record.service_id).unwrap().add_exception(record);
    }

    let services = services.into_iter()
        .map(|(id, service)| (id, Rc::new(service)))
        .collect();

    Ok(services)
}

#[derive(Debug, Deserialize)]
struct ServiceRecord {
    service_id: Id,
    #[serde(deserialize_with = "deserialize_naive_date")]
    start_date: NaiveDate,
    #[serde(deserialize_with = "deserialize_naive_date")]
    end_date: NaiveDate,
    #[serde(deserialize_with = "deserialize_numeric_bool")]
    monday: bool,
    #[serde(deserialize_with = "deserialize_numeric_bool")]
    tuesday: bool,
    #[serde(deserialize_with = "deserialize_numeric_bool")]
    wednesday: bool,
    #[serde(deserialize_with = "deserialize_numeric_bool")]
    thursday: bool,
    #[serde(deserialize_with = "deserialize_numeric_bool")]
    friday: bool,
    #[serde(deserialize_with = "deserialize_numeric_bool")]
    saturday: bool,
    #[serde(deserialize_with = "deserialize_numeric_bool")]
    sunday: bool,
}

#[derive(Debug)]
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
    #[serde(deserialize_with = "deserialize_naive_date")]
    date: NaiveDate,
    exception_type: ServiceExceptionType,
}
