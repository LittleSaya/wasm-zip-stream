use std::{cell::RefCell, rc::Rc};

use crate::prelude::*;

pub struct ManagerContext {
  pub window        : Rc<web_sys::Window>,
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
  pub fn new(create_writer: js_sys::Function) -> Rc<ManagerContext> {
    let window = web_sys::window().unwrap();
    Rc::new(
      Self {
        window        : Rc::new(window.clone()),
        performance   : Rc::new(window.performance().unwrap()),
        entry_list    : Rc::new(RefCell::new(Vec::new())),
        create_writer : Rc::new(create_writer),
      }
    )
  }
}
