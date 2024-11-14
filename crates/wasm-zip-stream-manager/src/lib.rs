use std::{mem, rc::Rc};

use context::Context;
use handles::Handles;
use manager_context::ManagerContext;
use manager_handles::ManagerHandles;
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
mod manager_handles;
mod manager_context;

#[wasm_bindgen(start)]
pub fn start() {
  utils::set_panic_hook();
}

#[wasm_bindgen]
pub fn initialize_context(create_writer: js_sys::Function) -> Handles {
  Context::init(create_writer)
}

#[wasm_bindgen]
pub fn create_manager(create_writer: js_sys::Function, worker_path: String) -> ManagerHandles {
  let context = ManagerContext::new(create_writer);
  let handles = ManagerHandles::new(Rc::clone(&context), worker_path);
  mem::forget(context);
  handles
}
