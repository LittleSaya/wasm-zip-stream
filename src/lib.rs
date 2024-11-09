use context::Context;
use handles::Handles;
use wasm_error::WasmError;
use prelude::*;

mod utils;
mod appnote63;
mod context;
mod prelude;
mod constant;
mod js_futures;
mod wasm_error;
mod handles;
mod transform_writer;
mod recover_writer;

#[wasm_bindgen]
pub fn initialize_context(create_writer: js_sys::Function) -> Handles {
  utils::set_panic_hook();
  Context::init(create_writer)
}
