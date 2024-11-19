use wasm_bindgen_futures::wasm_bindgen;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
  pub type GenericMessageData;

  #[wasm_bindgen(method, getter)]
  pub fn generic_message_type(this: &GenericMessageData) -> String;
}

pub const GENERIC_MESSAGE_TYPE_WORKER: &'static str = "worker";

#[wasm_bindgen]
extern "C" {
  pub type WorkerMessageData;

  #[wasm_bindgen(method, getter)]
  pub fn worker_message_type(this: &WorkerMessageData) -> String;
}

pub const WORKER_MESSAGE_TYPE_LOADED: &'static str = "loaded";

#[wasm_bindgen]
extern "C" {
  pub type WorkerLoadedMessageData;
}

pub const WORKER_MESSAGE_TYPE_INITIALIZE_WASM: &'static str = "initialize_wasm";

#[wasm_bindgen]
extern "C" {
  pub type WorkerInitializeWasmMessageData;
}
