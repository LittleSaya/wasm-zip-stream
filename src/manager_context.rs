use std::{cell::RefCell, mem, rc::Rc};

use crate::manager_handles::ManagerHandles;
use crate::prelude::*;

pub struct ManagerContext {
  pub performance   : Rc<web_sys::Performance>,
  pub entry_list    : Rc<RefCell<Vec<ZipEntry>>>,
  pub create_writer : Rc<js_sys::Function>,
}

pub struct ZipEntry {
  pub path   : String,
  pub is_dir : bool,
  pub file   : Option<web_sys::File>,
}

impl ManagerContext {
  pub fn new(create_writer: js_sys::Function) -> ManagerHandles {
    let window = web_sys::window().unwrap();

    let context = Rc::new(
      Self {
        performance   : Rc::new(window.performance().unwrap()),
        entry_list    : Rc::new(RefCell::new(Vec::new())),
        create_writer : Rc::new(create_writer),
      }
    );

    let handles = ManagerHandles::new(Rc::clone(&context));

    mem::forget(context);

    handles
  }
}
