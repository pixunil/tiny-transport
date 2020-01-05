use std::error::Error;
use std::rc::Rc;
use std::collections::HashMap;
use std::time::Instant;

use crate::utils::{Dataset, progress::elapsed};
use super::{Service, ServiceId, ServiceRecord, ServiceExceptionRecord};

pub(crate) struct Importer;

impl Importer {
    pub(crate) fn import(dataset: &mut impl Dataset) -> Result<HashMap<ServiceId, Rc<Service>>, Box<dyn Error>> {
        let mut services = Self::import_services(dataset)?;
        Self::add_service_exceptions(dataset, &mut services)?;

        let services = services.into_iter()
            .map(|(id, service)| (id, Rc::new(service)))
            .collect();

        Ok(services)
    }

    fn import_services(dataset: &mut impl Dataset) -> Result<HashMap<ServiceId, Service>, Box<dyn Error>> {
        let mut services = HashMap::new();

        let records = dataset.read_csv("calendar.txt", "Importing services")?;
        let started = Instant::now();
        for result in records {
            let record: ServiceRecord = result?;
            record.import(&mut services);
        }

        eprintln!("Imported {} services in {:.2}s", services.len(), elapsed(started));
        Ok(services)
    }

    fn add_service_exceptions(dataset: &mut impl Dataset, services: &mut HashMap<ServiceId, Service>) -> Result<(), Box<dyn Error>> {
        let records = dataset.read_csv("calendar_dates.txt", "Importing service exceptions")?;
        let started = Instant::now();
        for result in records {
            let record: ServiceExceptionRecord = result?;
            record.apply_to(services);
        }

        eprintln!("Imported service exceptions in {:.2}s", elapsed(started));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::NaiveDate;

    use crate::{map, dataset};
    use crate::service::fixtures::*;

    #[test]
    fn test_from_csv() {
        let mut dataset = dataset!(
            calendar:
                service_id, monday, tuesday, wednesday, thursday, friday, saturday, sunday, start_date, end_date;
                1,          1,      1,       1,         1,        1,      0,        0,      20190101,   20191231
            calendar_dates:
                service_id, date,     exception_type;
                1,          20190105, 1;
                1,          20190107, 2
        );

        let mut service = services::mon_fri();
        service.add_date(NaiveDate::from_ymd(2019, 1, 5));
        service.remove_date(NaiveDate::from_ymd(2019, 1, 7));

        let services = Importer::import(&mut dataset).unwrap();
        assert_eq!(services, map! {
            "1" => Rc::new(service),
        });
    }
}
