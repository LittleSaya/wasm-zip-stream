use std::rc::Rc;

use crate::{manager_context::{ManagerContext, ZipEntry}, prelude::*, utils, wasm_error::WasmError};

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = ["window", "wzs"])]
  fn file_system_file_entry__file(file_entry: web_sys::FileSystemFileEntry) -> js_sys::Promise;

  #[wasm_bindgen(js_namespace = ["window", "wzs"])]
  fn file_system_directory_reader__read_entries(directory_reader: &web_sys::FileSystemDirectoryReader) -> js_sys::Promise;

  #[wasm_bindgen(js_namespace = ["window", "wzs", "promises"], js_name = create)]
  fn create_promise(id: &str) -> js_sys::Promise;

  #[wasm_bindgen(js_namespace = ["window", "wzs", "promises"], js_name = resolve)]
  fn resolve_promise(id: &str);

  #[wasm_bindgen(js_namespace = ["window", "wzs", "promises"], js_name = reject)]
  fn reject_promise(id: &str, err: &JsValue);
}

const PROMISE_ID_WORKER_LOADED: &'static str = "worker_loaded";

#[wasm_bindgen]
extern "C" {
  pub type GenericMessageData;

  #[wasm_bindgen(method, getter)]
  pub fn generic_message_type(this: &GenericMessageData) -> String;
}

const GENERIC_MESSAGE_TYPE_WORKER: &'static str = "worker";

#[wasm_bindgen]
extern "C" {
  pub type WorkerMessageData;

  #[wasm_bindgen(method, getter)]
  pub fn worker_message_type(this: &WorkerMessageData) -> String;
}

const WORKER_MESSAGE_TYPE_LOADED: &'static str = "loaded";

#[wasm_bindgen]
pub struct ManagerHandles {
  context       : Rc<ManagerContext>,
  worker_path   : String,
  big_workers   : Vec<web_sys::Worker>,
  small_workers : Vec<web_sys::Worker>,
  scan_progress : Option<js_sys::Function>,
}

impl ManagerHandles {
  pub fn new(context: Rc<ManagerContext>, worker_path: String) -> Self {
    Self {
      context,
      worker_path,
      big_workers: Vec::new(),
      small_workers: Vec::new(),
      scan_progress: None,
    }
  }
}

#[wasm_bindgen]
impl ManagerHandles {
  /// Do a deep scan on input entries.
  ///
  /// # Parameters
  ///
  /// * `entries` - MUST be an array of `FileSystemEntry`
  ///
  /// # Returns
  ///
  /// - resolve : number of scanned entries
  /// - reject  : a `WasmError` object
  pub async fn scan(&self, entries: js_sys::Array) -> Result<JsValue, WasmError> {
    self.context.entry_list.borrow_mut().clear();

    self.scan_internal(entries).await?;

    Ok(JsValue::from_f64(self.context.entry_list.borrow().len() as f64))
  }

  pub async fn compress(&mut self, number_of_workers: u32) -> Result<JsValue, WasmError> {
    #[allow(non_snake_case)]
    let LOCATION = utils::type_name(&Self::compress);

    if self.big_workers.len() != 0 || self.small_workers.len() != 0 {
      return Err(WasmError::workers_not_cleaned(LOCATION, &format!("{}", self.big_workers.len()), &format!("{}", self.small_workers.len())));
    }

    if number_of_workers == 0 {
      return Err(WasmError::no_workers(LOCATION));
    }

    let number_of_small_workers = number_of_workers / 2_u32;
    let number_of_big_workers = number_of_workers - number_of_small_workers;

    // load workers
    {
      let mut ready_count = 0_u32;
      let ready_total = number_of_workers;
      let message_handler = Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |ev: web_sys::MessageEvent| {
        let data: GenericMessageData = ev.data().unchecked_into();
        let generic_message_type = data.generic_message_type();
        if generic_message_type == GENERIC_MESSAGE_TYPE_WORKER {
          let data: WorkerMessageData = data.unchecked_into();
          let worker_message_type = data.worker_message_type();
          if worker_message_type == WORKER_MESSAGE_TYPE_LOADED {
            ready_count += 1;
            if ready_count == ready_total {
              resolve_promise(PROMISE_ID_WORKER_LOADED);
            }
          }
        }
      }).into_js_value();

      for i in 0..number_of_big_workers {
        let worker_option = web_sys::WorkerOptions::new();
        worker_option.set_name(&format!("big_worker_{:0>3}", i));
        worker_option.set_type(web_sys::WorkerType::Module);
        let worker = match web_sys::Worker::new_with_options(&self.worker_path, &worker_option) {
          Ok(w) => w,
          Err(e) => return Err(WasmError::fail_to_create_worker(LOCATION, &format!("{:?}", e))),
        };

        if let Err(e) = worker.add_event_listener_with_callback("message", message_handler.unchecked_ref()) {
          return Err(WasmError::fail_to_listen_event(LOCATION, "message", &format!("{:?}", e)));
        }

        self.big_workers.push(worker);
      }

      for i in 0..number_of_small_workers {
        let worker_option = web_sys::WorkerOptions::new();
        worker_option.set_name(&format!("small_worker_{:0>3}", i));
        worker_option.set_type(web_sys::WorkerType::Module);
        let worker = match web_sys::Worker::new_with_options(&self.worker_path, &worker_option) {
          Ok(w) => w,
          Err(e) => return Err(WasmError::fail_to_create_worker(LOCATION, &format!("{:?}", e))),
        };

        if let Err(e) = worker.add_event_listener_with_callback("message", message_handler.unchecked_ref()) {
          return Err(WasmError::fail_to_listen_event(LOCATION, "message", &format!("{:?}", e)));
        }

        self.small_workers.push(worker);
      }

      utils::await_promise(create_promise(PROMISE_ID_WORKER_LOADED)).await.unwrap();

      for worker in self.big_workers.iter() {
        if let Err(e) = worker.remove_event_listener_with_callback("message", message_handler.unchecked_ref()) {
          return Err(WasmError::fail_to_unlisten_event(LOCATION, "message", &format!("{:?}", e)));
        }
      }

      for worker in self.small_workers.iter() {
        if let Err(e) = worker.remove_event_listener_with_callback("message", message_handler.unchecked_ref()) {
          return Err(WasmError::fail_to_unlisten_event(LOCATION, "message", &format!("{:?}", e)));
        }
      }
    }

    Ok(JsValue::UNDEFINED)
  }

  /// Register "scan_progress" callback.
  ///
  /// # Parameters
  ///
  /// * `callback` - a function like `(number_of_scanned_entries: number) => {}`
  pub fn register_scan_progress(&mut self, callback: js_sys::Function) {
    self.scan_progress = Some(callback);
  }
}

/// Recursively scan, populate `entry_list` in `ManagerContext`.
impl ManagerHandles {
  async fn scan_internal(&self, entries: js_sys::Array) -> Result<JsValue, WasmError> {
    #[allow(non_snake_case)]
    let LOCATION = utils::type_name(&Self::scan_internal);

    // cast elements into FileSystemEntries
    let array_length = entries.length();
    let mut temp_vector = Vec::<web_sys::FileSystemEntry>::with_capacity(array_length as usize);
    for i in 0..array_length {
      temp_vector.push(entries.get(i).unchecked_into::<web_sys::FileSystemEntry>());
    }

    let entries = temp_vector;
    for entry in entries {
      let full_path = entry.full_path();

      if entry.is_file() {
        let file_entry = entry.unchecked_into::<web_sys::FileSystemFileEntry>();

        let file = match utils::await_promise(file_system_file_entry__file(file_entry)).await {
          Ok(f) => match f.dyn_into::<web_sys::File>() {
            Ok(f) => f,
            Err(_) => return Err(WasmError::dynamic_cast_error(LOCATION, "JsValue", "File")),
          },
          Err(e) => return Err(WasmError::fail_to_get_file(LOCATION, &format!("{:?}", e))),
        };

        self.context.entry_list.borrow_mut().push(ZipEntry { path: full_path, is_dir: false, file: Some(file) });

        self.report_scan_progress(self.context.entry_list.borrow().len())?;
      }
      else if entry.is_directory() {
        let directory_entry = entry.unchecked_into::<web_sys::FileSystemDirectoryEntry>();

        self.context.entry_list.borrow_mut().push(ZipEntry { path: full_path, is_dir: true, file: None });

        self.report_scan_progress(self.context.entry_list.borrow().len())?;

        let directory_reader = directory_entry.create_reader();

        let mut child_entries = js_sys::Array::new();
        loop {
          let partial = match utils::await_promise(file_system_directory_reader__read_entries(&directory_reader)).await {
            Ok(arr) => match arr.dyn_into::<js_sys::Array>() {
              Ok(arr) => arr,
              Err(_) => return Err(WasmError::dynamic_cast_error(LOCATION, "JsValue", "Array")),
            },
            Err(e) => return Err(WasmError::fail_to_get_file_entry(LOCATION, &format!("{:?}", e))),
          };

          if partial.length() > 0 {
            child_entries = child_entries.concat(&partial);
          } else {
            break;
          }
        }

        if let Err(e) = Box::pin(self.scan_internal(child_entries)).await {
          return Err(e);
        }
      }
      else {
        return Err(WasmError::unknown_file_entry(LOCATION));
      }
    }

    Ok(JsValue::UNDEFINED)
  }

  fn report_scan_progress(&self, num: usize) -> Result<(), WasmError> {
    if let Some(scan_progress) = self.scan_progress.as_ref() {
      if let Err(e) = scan_progress.call1(&JsValue::NULL, &JsValue::from_f64(num as f64)) {
        return Err(WasmError::fail_to_invoke_callback(utils::type_name(&Self::report_scan_progress), "scan_progress", &format!("{:?}", e)));
      }
    }
    Ok(())
  }
}
