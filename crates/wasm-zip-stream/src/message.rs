use wasm_bindgen_futures::wasm_bindgen;
use wasm_bindgen::prelude::*;
use web_sys::js_sys::{self, Reflect};

use crate::WasmError;

const F_GENERIC_MESSAGE_TYPE: &'static str = "generic_message_type";
const F_WORKER_MESSAGE_TYPE: &'static str = "worker_message_type";

pub fn type_name<T: ?Sized>(_val: &T) -> &'static str {
  std::any::type_name::<T>()
}

// ----- GenericMessageData

#[wasm_bindgen]
extern "C" {
  pub type GenericMessageData;

  #[wasm_bindgen(method, getter)]
  pub fn generic_message_type(this: &GenericMessageData) -> String;
}

pub const GENERIC_MESSAGE_TYPE_WORKER: &'static str = "worker";

// ----- ----- WorkerMessageData

#[wasm_bindgen]
extern "C" {
  pub type WorkerMessageData;

  #[wasm_bindgen(method, getter)]
  pub fn worker_message_type(this: &WorkerMessageData) -> String;
}

pub const WORKER_MESSAGE_TYPE_LOADED: &'static str = "loaded";
pub const WORKER_MESSAGE_TYPE_INITIALIZE_WASM: &'static str = "initialize_wasm";
pub const WORKER_MESSAGE_TYPE_INITIALIZE_WASM_SUCCESS: &'static str = "initialize_wasm_success";
pub const WORKER_MESSAGE_TYPE_INITIALIZE_WASM_FAIL: &'static str = "initialize_wasm_fail";

// ----- ----- ----- WorkerLoadedMessageData

#[wasm_bindgen]
extern "C" {
  pub type WorkerLoadedMessageData;
}

// ----- ----- ----- WorkerInitializeWasmMessageData

#[wasm_bindgen]
extern "C" {
  pub type WorkerInitializeWasmMessageData;
}

impl WorkerInitializeWasmMessageData {
  pub fn new(worker_wasm_path: &str) -> Result<Self, WasmError> {
    #[allow(non_snake_case)]
    let LOCATION = type_name(&Self::new);

    let msg = js_sys::Object::new();

    if let Err(e) = Reflect::set(&msg, &JsValue::from_str(F_GENERIC_MESSAGE_TYPE), &JsValue::from_str(GENERIC_MESSAGE_TYPE_WORKER)) {
      return Err(WasmError::fail_to_set_property(LOCATION, F_GENERIC_MESSAGE_TYPE, &format!("{:?}", e)));
    }

    if let Err(e) = Reflect::set(&msg, &JsValue::from_str(F_WORKER_MESSAGE_TYPE), &JsValue::from_str(WORKER_MESSAGE_TYPE_INITIALIZE_WASM)) {
      return Err(WasmError::fail_to_set_property(LOCATION, F_WORKER_MESSAGE_TYPE, &format!("{:?}", e)));
    }

    if let Err(e) = Reflect::set(&msg, &JsValue::from_str("worker_wasm_path"), &JsValue::from_str(worker_wasm_path)) {
      return Err(WasmError::fail_to_set_property(LOCATION, "worker_wasm_path", &format!("{:?}", e)));
    }

    Ok(msg.unchecked_into())
  }
}

// ----- ----- ----- WorkerInitializeWasmSuccessMessageData

#[wasm_bindgen]
extern "C" {
  pub type WorkerInitializeWasmSuccessMessageData;
}

// ----- ----- ----- WorkerInitializeWasmFailMessageData

#[wasm_bindgen]
extern "C" {
  pub type WorkerInitializeWasmFailMessageData;

  #[wasm_bindgen(method, getter)]
  pub fn detail(this: &WorkerInitializeWasmFailMessageData) -> JsValue;
}
