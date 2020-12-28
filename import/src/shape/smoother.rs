use std::error::Error;
use std::fmt;
use std::str::FromStr;

use crate::coord::Point;
use crate::shape::Buffer;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    Off,
    Deduplicate,
    Full,
}

impl Mode {
    pub(super) fn smooth(self, shape: Buffer) -> Buffer {
        let mut smoother = Smoother::new(self);
        for position in shape {
            smoother.add(position);
        }
        smoother.finish()
    }

    pub fn variants() -> &'static [&'static str] {
        &["off", "deduplicate", "full"]
    }
}

#[derive(Debug, Clone)]
pub struct InvalidModeError(String);

impl fmt::Display for InvalidModeError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "smoothing mode '{}' not found", self.0)
    }
}

impl Error for InvalidModeError {}

impl FromStr for Mode {
    type Err = InvalidModeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "off" => Ok(Self::Off),
            "deduplicate" => Ok(Self::Deduplicate),
            "full" => Ok(Self::Full),
            _ => Err(InvalidModeError(value.to_string())),
        }
    }
}

#[derive(Debug)]
struct Smoother {
    mode: Mode,
    points: Vec<Point>,
    last_sharp_turn: Option<usize>,
}

impl Smoother {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            points: Vec::new(),
            last_sharp_turn: None,
        }
    }

    fn dedup(&mut self) -> bool {
        let len = self.points.len();
        if len >= 2 && self.points[len - 2] == self.points[len - 1] {
            self.points.pop();
            return true;
        }
        false
    }

    fn remove_spike(&mut self, at: usize) -> bool {
        let spike_angle = 80_f64.to_radians();
        if at < 2 {
            return false;
        }
        if let Some(neighborhood) = self.points.get(at - 2..=at + 2) {
            let segments = [
                neighborhood[1] - neighborhood[0],
                neighborhood[2] - neighborhood[1],
                neighborhood[3] - neighborhood[2],
                neighborhood[4] - neighborhood[3],
            ];
            let angles = [
                segments[0].angle(&segments[1]),
                segments[1].angle(&segments[2]),
                segments[2].angle(&segments[3]),
            ];

            if angles[1] > spike_angle
                && angles[0] + angles[2] > spike_angle
                && angles[1] > angles[2]
            {
                if neighborhood[1] == neighborhood[3] {
                    self.points.drain(at..=at + 1);
                } else {
                    self.points.remove(at);
                }
                self.last_sharp_turn = None;
                return true;
            }
        }
        false
    }

    fn correct_reversed_part(&mut self, at: usize) -> bool {
        let sharp_turn_angle = 160_f64.to_radians();
        if at < 1 {
            return false;
        }
        if let Some(neighborhood) = self.points.get(at - 1..=at + 1) {
            let segments = [
                neighborhood[1] - neighborhood[0],
                neighborhood[2] - neighborhood[1],
            ];
            if segments[0].angle(&segments[1]) >= sharp_turn_angle {
                match self.last_sharp_turn {
                    Some(last_sharp_turn) if at - last_sharp_turn >= 3 => {
                        self.points[last_sharp_turn..=at].reverse();
                        for offset in -1..=1 {
                            if self.remove_spike((last_sharp_turn as isize + offset) as usize) {
                                break;
                            }
                        }
                        for offset in -1..=1 {
                            if self.remove_spike((at as isize + offset) as usize) {
                                break;
                            }
                        }
                        self.last_sharp_turn = None;
                        return true;
                    }
                    _ => self.last_sharp_turn = Some(at),
                }
            }
        }

        false
    }

    fn add(&mut self, point: Point) {
        self.points.push(point);

        match self.mode {
            Mode::Off => {}
            Mode::Deduplicate => {
                self.dedup();
            }
            Mode::Full => {
                if self.dedup() {
                    return;
                }
                while self.remove_spike(self.points.len().saturating_sub(3)) {}
                self.correct_reversed_part(self.points.len().saturating_sub(4));
            }
        }
    }

    fn finish(mut self) -> Buffer {
        self.correct_reversed_part(self.points.len().saturating_sub(3));
        self.correct_reversed_part(self.points.len().saturating_sub(2));
        Buffer::from(self.points)
    }
}

#[cfg(test)]
mod tests {
    macro_rules! test_smoothing {
        (@fixtures $name:ident,
            { $( $variant:ident : [ $( $lat:expr, $lon:expr );* $(;)? ] ),* $(,)? }
        ) => {
            pub(in crate::shape) mod $name {
                use crate::coord::project;
                use crate::shape::Buffer;

                $(
                    pub(in crate::shape) fn $variant() -> Buffer {
                        Buffer::from(vec![$( project($lat, $lon) ),*])
                    }
                )*
                test_smoothing!(@aliases [ $( $variant ),* ]);
            }
        };
        (@aliases [unprocessed, deduplicated]) => {
            pub(in crate::shape) fn corrected() -> Buffer { deduplicated() }
        };
        (@aliases [unprocessed, corrected]) => {
            pub(in crate::shape) fn deduplicated() -> Buffer { unprocessed() }
        };
        (@aliases [unprocessed, deduplicated, corrected]) => {};
        (@tests $name:ident, $mode:ident, $expected:ident) => {
            mod $name {
                use super::super::fixtures::$name as variants;
                use crate::shape::smoother::Mode;
                use common::assert_eq_alternate;

                #[test]
                fn upstream() {
                    assert_eq_alternate!(
                        Mode::$mode.smooth(variants::unprocessed()),
                        variants::$expected(),
                    );
                }

                #[test]
                fn upstream_idempotence() {
                    assert_eq_alternate!(
                        Mode::$mode.smooth(variants::$expected()),
                        variants::$expected(),
                    );
                }

                #[test]
                fn downstream() {
                    assert_eq_alternate!(
                        Mode::$mode.smooth(variants::unprocessed().reversed()),
                        variants::$expected().reversed(),
                    );
                }

                #[test]
                fn downstream_idempotence() {
                    assert_eq_alternate!(
                        Mode::$mode.smooth(variants::$expected().reversed()),
                        variants::$expected().reversed(),
                    );
                }
            }
        };
        ($($name:ident : $shapes:tt),* $(,)?) => {
            mod fixtures {
                $( test_smoothing!(@fixtures $name, $shapes); )*
            }
            mod deduplicate {
                $( test_smoothing!(@tests $name, Deduplicate, deduplicated); )*
            }
            mod full {
                $( test_smoothing!(@tests $name, Full, corrected); )*
            }
        };
    }

    test_smoothing! {
        borgsdorf: {
            unprocessed: [
    52.713640, 13.277353; 52.714390, 13.276990; 52.714697, 13.276836; 52.714491, 13.276771; 52.714491, 13.276771;
    52.714516, 13.276934; 52.714697, 13.276836; 52.715103, 13.276641;
            ],
            deduplicated: [
    52.713640, 13.277353; 52.714390, 13.276990; 52.714697, 13.276836; 52.714491, 13.276771; 52.714516, 13.276934;
    52.714697, 13.276836; 52.715103, 13.276641;
            ],
            corrected: [
    52.713640, 13.277353; 52.714390, 13.276990; 52.714516, 13.276934; 52.714697, 13.276836; 52.715103, 13.276641;
            ],
        },
        schoenholz: {
            unprocessed: [
    52.570339, 13.383202; 52.570881, 13.382216; 52.571386, 13.381303; 52.571734, 13.380665; 52.571734, 13.380665;
    52.571734, 13.380665; 52.571854, 13.380447; 52.572037, 13.380158;
            ],
            deduplicated: [
    52.570339, 13.383202; 52.570881, 13.382216; 52.571386, 13.381303; 52.571734, 13.380665; 52.571854, 13.380447;
    52.572037, 13.380158;
            ],
        },
        gesundbrunnen_bornholmer_str: {
            unprocessed: [
    52.552942, 13.398327; 52.553239, 13.398248; 52.553554, 13.398185; 52.549073, 13.388017; 52.553924, 13.398049;
    52.554204, 13.397969; 52.554618, 13.397894;
            ],
            corrected: [
    52.552942, 13.398327; 52.553239, 13.398248; 52.553554, 13.398185; 52.553924, 13.398049; 52.554204, 13.397969;
    52.554618, 13.397894;
            ],
        },
        westend: {
            unprocessed: [
    52.506269, 13.282699; 52.506368, 13.282732; 52.506422, 13.282748; 52.506484, 13.282764; 52.506627, 13.282813;
    52.507790, 13.283458; 52.506645, 13.282828; 52.506851, 13.282937; 52.507119, 13.283063; 52.507538, 13.283355;
    52.507779, 13.283509; 52.508091, 13.283739; 52.508296, 13.283892; 52.508769, 13.284274; 52.509428, 13.284794;
    52.509999, 13.285209; 52.510088, 13.285270; 52.510159, 13.285302; 52.510240, 13.285363; 52.510329, 13.285410;
    52.510481, 13.285517; 52.510570, 13.285564; 52.510928, 13.285722; 52.511240, 13.285893; 52.511491, 13.285989;
    52.511769, 13.286041; 52.511948, 13.286076; 52.512253, 13.286129; 52.512505, 13.286151; 52.512936, 13.286164;
    52.513557, 13.286049; 52.513737, 13.286025; 52.513981, 13.285944; 52.514883, 13.285631; 52.515425, 13.285411;
    52.516128, 13.285152; 52.516553, 13.284973; 52.517599, 13.284620; 52.518203, 13.284447; 52.517892, 13.284394;
            ],
            corrected: [
    52.506269, 13.282699; 52.506368, 13.282732; 52.506422, 13.282748; 52.506484, 13.282764; 52.506627, 13.282813;
    52.506645, 13.282828; 52.506851, 13.282937; 52.507119, 13.283063; 52.507538, 13.283355;
    52.507779, 13.283509; 52.508091, 13.283739; 52.508296, 13.283892; 52.508769, 13.284274; 52.509428, 13.284794;
    52.509999, 13.285209; 52.510088, 13.285270; 52.510159, 13.285302; 52.510240, 13.285363; 52.510329, 13.285410;
    52.510481, 13.285517; 52.510570, 13.285564; 52.510928, 13.285722; 52.511240, 13.285893; 52.511491, 13.285989;
    52.511769, 13.286041; 52.511948, 13.286076; 52.512253, 13.286129; 52.512505, 13.286151; 52.512936, 13.286164;
    52.513557, 13.286049; 52.513737, 13.286025; 52.513981, 13.285944; 52.514883, 13.285631; 52.515425, 13.285411;
    52.516128, 13.285152; 52.516553, 13.284973; 52.517599, 13.284620; 52.518203, 13.284447; 52.517892, 13.284394;
            ],
        },
        feuerbachstr: {
            unprocessed: [
    52.461324, 13.329917; 52.462734, 13.331607; 52.463513, 13.332602; 52.463431, 13.332339; 52.463431, 13.332339;
    52.463974, 13.333190; 52.464054, 13.333295;
            ],
            deduplicated: [
    52.461324, 13.329917; 52.462734, 13.331607; 52.463513, 13.332602; 52.463431, 13.332339; 52.463974, 13.333190;
    52.464054, 13.333295;
            ],
            corrected: [
    52.461324, 13.329917; 52.462734, 13.331607; 52.463513, 13.332602; 52.463974, 13.333190; 52.464054, 13.333295;
            ],
        },
        botanischer_garten: {
            unprocessed: [
    52.447535, 13.305967; 52.447595, 13.306160; 52.447552, 13.306041; 52.447552, 13.306041; 52.447552, 13.306041;
    52.447595, 13.306160; 52.448118, 13.307646; 52.448144, 13.307735;
            ],
            deduplicated: [
    52.447535, 13.305967; 52.447595, 13.306160; 52.447552, 13.306041; 52.447595, 13.306160; 52.448118, 13.307646;
    52.448144, 13.307735;
            ],
            corrected: [
    52.447535, 13.305967; 52.447595, 13.306160; 52.448118, 13.307646; 52.448144, 13.307735;
            ],
        },
        schlachtensee: {
            unprocessed: [
    52.439063, 13.210216; 52.439191, 13.210866; 52.439593, 13.212863; 52.439825, 13.213972; 52.439806, 13.213986;
    52.439806, 13.213986; 52.440321, 13.216383; 52.440475, 13.217152; 52.440611, 13.217906;
            ],
            deduplicated: [
    52.439063, 13.210216; 52.439191, 13.210866; 52.439593, 13.212863; 52.439825, 13.213972; 52.439806, 13.213986;
    52.440321, 13.216383; 52.440475, 13.217152; 52.440611, 13.217906;
            ],
            corrected: [
    52.439063, 13.210216; 52.439191, 13.210866; 52.439593, 13.212863; 52.439825, 13.213972; 52.440321, 13.216383;
    52.440475, 13.217152; 52.440611, 13.217906;
            ],
        },
        wannsee: {
            unprocessed: [
    52.421315, 13.179801; 52.431773, 13.194295; 52.430077, 13.192271; 52.428630, 13.190667; 52.426442, 13.188091;
    52.424942, 13.186370; 52.424363, 13.185603; 52.431798, 13.194071; 52.431798, 13.194071; 52.431838, 13.194226;
    52.431909, 13.194302; 52.432229, 13.194663; 52.432567, 13.195084; 52.432815, 13.195414; 52.433284, 13.196103;
            ],
            deduplicated: [
    52.421315, 13.179801; 52.431773, 13.194295; 52.430077, 13.192271; 52.428630, 13.190667; 52.426442, 13.188091;
    52.424942, 13.186370; 52.424363, 13.185603; 52.431798, 13.194071; 52.431838, 13.194226; 52.431909, 13.194302;
    52.432229, 13.194663; 52.432567, 13.195084; 52.432815, 13.195414; 52.433284, 13.196103;
            ],
            corrected: [
    52.421315, 13.179801; 52.424363, 13.185603; 52.424942, 13.186370; 52.426442, 13.188091; 52.428630, 13.190667;
    52.430077, 13.192271; 52.431773, 13.194295; 52.431838, 13.194226; 52.431909, 13.194302; 52.432229, 13.194663;
    52.432567, 13.195084; 52.432815, 13.195414; 52.433284, 13.196103;
            ],
        },
    }
}
