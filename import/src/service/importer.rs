use std::error::Error;
use std::rc::Rc;
use std::collections::HashMap;

use crate::utils::Dataset;
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
        let mut reader = dataset.read_csv("calendar.txt")?;
        for result in reader.deserialize() {
            let record: ServiceRecord = result?;
            record.import(&mut services);
        }
        Ok(services)
    }

    fn add_service_exceptions(dataset: &mut impl Dataset, services: &mut HashMap<ServiceId, Service>) -> Result<(), Box<dyn Error>> {
        let mut reader = dataset.read_csv("calendar_dates.txt")?;
        for result in reader.deserialize() {
            let record: ServiceExceptionRecord = result?;
            record.apply_to(services);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::NaiveDate;

    #[macro_export]
    macro_rules! service {
        ($start:expr, $end:expr, $weekdays:expr) => ({
            let start = chrono::NaiveDate::from_ymd($start[0], $start[1], $start[2]);
            let end = chrono::NaiveDate::from_ymd($end[0], $end[1], $end[2]);
            crate::service::Service::new(start, end, $weekdays)
        });
        (mon-fri) => (
            $crate::service!([2019, 1, 1], [2019, 12, 31], [true, true, true, true, true, false, false])
        );
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
        service.add_date(NaiveDate::from_ymd(2019, 1, 5));
        service.remove_date(NaiveDate::from_ymd(2019, 1, 7));

        let services = Importer::import(&mut dataset).unwrap();
        assert_eq!(services.len(), 1);
        assert_eq!(*services[&"1".into()], service);
    }
}
