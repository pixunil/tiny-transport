use serde_derive::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Schedule {
    driving_durations: Vec<(u32, u32)>,
    stop_duration_at_terminus: u32,
}

impl Schedule {
    pub fn new(driving_durations: Vec<(u32, u32)>, stop_duration_at_terminus: u32) -> Self {
        Self {
            driving_durations,
            stop_duration_at_terminus,
        }
    }

    pub fn stop_duration_at_terminus(&self) -> u32 {
        self.stop_duration_at_terminus
    }
}

impl<'a> IntoIterator for &'a Schedule {
    type Item = &'a (u32, u32);
    type IntoIter = <&'a Vec<(u32, u32)> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.driving_durations.iter()
    }
}

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    use itertools::Itertools;

    use super::*;
    use common::times;

    macro_rules! schedules {
        ($( $schedule:ident : [$( $( $(:)? $time:literal )* ),* ] ),* $(,)?) => (
            $(
                pub fn $schedule() -> Schedule {
                    let mut durations = times!($($(: $time)*),*).into_iter().tuples();
                    let mut driving_durations = Vec::new();
                    while let Some((stopping, driving)) = durations.next() {
                        driving_durations.push((stopping, driving));
                    }
                    let stop_duration_at_terminus = durations.into_buffer().exactly_one().unwrap();
                    Schedule::new(driving_durations, stop_duration_at_terminus)
                }
            )*
        );
    }

    schedules! {
        hackescher_markt_bellevue:
        [0:30, 1:30, 0:48, 1:54, 0:36, 2:06, 0:30],
        bellevue_hackescher_markt:
        [0:30, 2:06, 0:42, 1:54, 0:48, 1:30, 0:30],
        naturkundemuseum_franzoesische_str:
        [0:00, 1:30, 0:00, 1:00, 0:00, 1:30, 0:00],
        franzoesische_str_naturkundemuseum:
        [0:00, 1:30, 0:00, 1:30, 0:00, 1:00, 0:00],
        zingster_str_prerower_platz:
        [0:00, 1:00, 0:00, 1:00, 0:00, 2:00, 0:00],
        oranienburger_tor_am_kupfergraben:
        [0:20, 2:15, 0:20, 1:05, 0:20, 1:00, 0:20],
        am_kupfergraben_oranienburger_tor:
        [0:20, 1:15, 0:20, 1:33, 0:20, 2:32, 0:20],
        weskammstr_waldsassener_str:
        [0:00, 0:30, 0:00, 1:00, 0:00],
    }
}
