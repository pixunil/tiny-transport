use std::error::Error;
use std::rc::Rc;
use std::fmt;
use std::collections::{HashSet, HashMap};

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

fn import_services(dataset: &mut impl Dataset) -> Result<HashMap<Id, Service>, Box<dyn Error>> {
    let mut services = HashMap::new();
    let mut reader = dataset.read_csv("calendar.txt")?;
    for result in reader.deserialize() {
        let (id, service) = Service::new(result?);
        services.insert(id, service);
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

pub fn from_csv(dataset: &mut impl Dataset) -> Result<HashMap<Id, Rc<Service>>, Box<dyn Error>> {
    let mut services = import_services(dataset)?;
    add_service_exceptions(dataset, &mut services)?;

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
