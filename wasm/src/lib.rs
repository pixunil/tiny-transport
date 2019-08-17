mod map;
mod view;

#[cfg(feature = "console_error_panic_hook")]
#[wasm_bindgen(start)]
pub fn main() {
    ::std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}
