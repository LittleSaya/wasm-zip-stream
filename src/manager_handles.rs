use std::rc::Rc;

use crate::{manager_context::{ManagerContext, ZipEntry}, prelude::*, utils, wasm_error::WasmError};

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = ["window", "wzs"])]
  fn file_system_file_entry__file(file_entry: web_sys::FileSystemFileEntry) -> js_sys::Promise;

  #[wasm_bindgen(js_namespace = ["window", "wzs"])]
  fn file_system_directory_reader__read_entries(directory_reader: &web_sys::FileSystemDirectoryReader) -> js_sys::Promise;
}

#[wasm_bindgen]
pub struct ManagerHandles {
  context       : Rc<ManagerContext>,
  scan_progress : Option<js_sys::Function>,
}

impl ManagerHandles {
  pub fn new(context: Rc<ManagerContext>) -> Self {
    Self {
      context,
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
