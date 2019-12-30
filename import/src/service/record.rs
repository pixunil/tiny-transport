use std::collections::HashMap;

use serde_derive::Deserialize;

use chrono::NaiveDate;

use crate::utils::deserialize;
use super::{Service, ServiceId, ExceptionType};

#[derive(Debug, Deserialize)]
pub(super) struct ServiceRecord {
    service_id: ServiceId,
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

impl ServiceRecord {
    pub(super) fn import(self, services: &mut HashMap<ServiceId, Service>) {
        let id = self.service_id.clone();
        services.insert(id, self.into());
    }
}

impl Into<Service> for ServiceRecord {
    fn into(self) -> Service {
        let weekdays = [self.monday, self.tuesday, self.wednesday, self.thursday, self.friday, self.saturday, self.sunday];
        Service::new(self.start_date, self.end_date, weekdays)
    }
}


#[derive(Debug, Deserialize)]
pub(super) struct ServiceExceptionRecord {
    service_id: ServiceId,
    #[serde(deserialize_with = "deserialize::naive_date")]
    date: NaiveDate,
    exception_type: ExceptionType,
}

impl ServiceExceptionRecord {
    pub(super) fn apply_to(self, services: &mut HashMap<ServiceId, Service>) {
        let service = services.get_mut(&self.service_id).unwrap();
        match self.exception_type {
            ExceptionType::Added => service.add_date(self.date),
            ExceptionType::Removed => service.remove_date(self.date),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{map, service};

    fn working_day_service_record() -> ServiceRecord {
        ServiceRecord {
            service_id: "1".into(),
            start_date: NaiveDate::from_ymd(2019, 1, 1),
            end_date: NaiveDate::from_ymd(2019, 12, 31),
            monday: true,
            tuesday: true,
            wednesday: true,
            thursday: true,
            friday: true,
            saturday: false,
            sunday: false,
        }
    }

    #[test]
    fn test_into_service() {
        let service: Service = working_day_service_record().into();
        assert_eq!(service, service!(mon-fri));
    }

    #[test]
    fn test_import() {
        let mut services = HashMap::new();
        working_day_service_record().import(&mut services);
        assert_eq!(services.len(), 1);
        assert_eq!(services[&"1".into()], service!(mon-fri));
    }

    fn services() -> HashMap<ServiceId, Service> {
        map! {
            "1" => service!(mon-fri),
        }
    }

    #[test]
    fn test_apply_include_exception() {
        let mut services = services();
        let record = ServiceExceptionRecord {
            service_id: "1".into(),
            date: NaiveDate::from_ymd(2019, 1, 5),
            exception_type: ExceptionType::Added,
        };
        record.apply_to(&mut services);
        assert!(services[&"1".into()].available_at(NaiveDate::from_ymd(2019, 1, 5)));
    }

    #[test]
    fn test_add_exclude_exception_to_service() {
        let mut services = services();
        let record = ServiceExceptionRecord {
            service_id: "1".into(),
            date: NaiveDate::from_ymd(2019, 12, 24),
            exception_type: ExceptionType::Removed,
        };
        record.apply_to(&mut services);
        assert!(!services[&"1".into()].available_at(NaiveDate::from_ymd(2019, 12, 24)));
    }
}
