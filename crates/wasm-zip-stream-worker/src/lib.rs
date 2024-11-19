mod prelude;
mod worker_handles;
mod utils;

use prelude::*;
use worker_handles::WorkerHandles;

#[wasm_bindgen(start)]
pub fn start() {
  utils::set_panic_hook();
}

#[wasm_bindgen]
pub fn create_worker() -> WorkerHandles {
  WorkerHandles {}
}
