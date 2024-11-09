//! The type `Context` works like a global object, data is shared between "scan" stage and "compress" stage.

use std::{cell::RefCell, mem, rc::Rc};

use crate::prelude::*;
use crate::handles::Handles;

pub struct Context {
  pub performance            : Rc<web_sys::Performance>,
  pub scan_stage             : Rc<ContextScanStage>,
}

pub struct ContextScanStage {
  pub file_system            : Rc<RefCell<Option<web_sys::FileSystem>>>,
  pub file_path_list         : Rc<RefCell<Vec<FilePath>>>,
}

pub struct FilePath {
  pub path   : String,
  pub is_dir : bool,
}

impl Context {
  pub fn init(create_writer: js_sys::Function) -> crate::Handles {
    let window = web_sys::window().unwrap();

    let context = Rc::new(
      Self {
        performance: Rc::new(window.performance().unwrap()),

        scan_stage: Rc::new(ContextScanStage {
          file_system            : Rc::new(RefCell::new(None)),
          file_path_list         : Rc::new(RefCell::new(Vec::new())),
        }),
      }
    );

    let handles = Handles::new(&context, create_writer);

    mem::forget(context);

    handles
  }
}
