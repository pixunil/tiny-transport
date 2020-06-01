mod dataset;
mod view;

#[cfg(feature = "console_error_panic_hook")]
mod panic_hook {
    use std::panic;

    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(start)]
    pub fn main() {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
    }
}
