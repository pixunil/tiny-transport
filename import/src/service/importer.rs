use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;

use super::{Service, ServiceExceptionRecord, ServiceId, ServiceRecord};
use crate::utils::{Action, Dataset};

pub(crate) struct Importer;

impl Importer {
    pub(crate) fn import(
        dataset: &mut impl Dataset,
    ) -> Result<HashMap<ServiceId, Rc<Service>>, Box<dyn Error>> {
        let mut services = Self::import_services(dataset)?;
        Self::add_service_exceptions(dataset, &mut services)?;

        let services = services
            .into_iter()
            .map(|(id, service)| (id, Rc::new(service)))
            .collect();

        Ok(services)
    }

    fn import_services(
        dataset: &mut impl Dataset,
    ) -> Result<HashMap<ServiceId, Service>, Box<dyn Error>> {
        let mut services = HashMap::new();

        let action = Action::start("Importing services");
        for result in action.read_csv(dataset, "calendar.txt")? {
            let record: ServiceRecord = result?;
            record.import(&mut services);
        }
        action.complete(&format!("Imported {} services", services.len()));
        Ok(services)
    }

    fn add_service_exceptions(
        dataset: &mut impl Dataset,
        services: &mut HashMap<ServiceId, Service>,
    ) -> Result<(), Box<dyn Error>> {
        let action = Action::start("Importing service exceptions");
        for result in action.read_csv(dataset, "calendar_dates.txt")? {
            let record: ServiceExceptionRecord = result?;
            record.apply_to(services);
        }
        action.complete("Imported service exceptions");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;
    use crate::dataset;
    use crate::fixtures::services;
    use test_utils::map;

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

        assert_eq!(
            Importer::import(&mut dataset).unwrap(),
            map! {
                "1" => Rc::new(service),
            }
        );
    }
}
