#[cfg(test)]
#[macro_use]
extern crate approx;
extern crate nalgebra as na;
extern crate gtfs_sim_simulation as simulation;
extern crate gtfs_sim_serialization as serialization;

mod map;
mod view;

#[cfg(feature = "console_error_panic_hook")]
#[wasm_bindgen(start)]
pub fn main() {
    ::std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}
