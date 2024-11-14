use std::{cell::RefCell, future::Future, pin::Pin, rc::Rc, task::{Context, Poll, Waker}};

use crate::prelude::*;

type SuccessType = web_sys::FileSystemFileEntry;
type ErrorType = web_sys::DomException;

struct Inner {
  result: Option<Result<SuccessType, ErrorType>>,
  task: Option<Waker>,
  callbacks: Option<(Closure<dyn FnMut(SuccessType)>, Closure<dyn FnMut(ErrorType)>)>,
}

#[allow(non_camel_case_types)]
pub struct FileSystemDirectoryEntry_getFile_future {
  inner: Rc<RefCell<Inner>>,
}

impl FileSystemDirectoryEntry_getFile_future {
  pub fn from(directory_entry: web_sys::FileSystemDirectoryEntry, path: &str) -> FileSystemDirectoryEntry_getFile_future {
    let state = Rc::new(RefCell::new(Inner {
      result: None,
      task: None,
      callbacks: None,
    }));

    fn finish(state: &RefCell<Inner>, val: Result<SuccessType, ErrorType>) {
      let task = {
        let mut state = state.borrow_mut();
        debug_assert!(state.callbacks.is_some());
        debug_assert!(state.result.is_none());

        drop(state.callbacks.take());

        state.result = Some(val);
        state.task.take()
      };

      if let Some(task) = task {
        task.wake()
      }
    }

    let success = {
      let state = state.clone();
      Closure::once(move |val| finish(&state, Ok(val)))
    };

    let error = {
      let state = state.clone();
      Closure::once(move |val| finish(&state, Err(val)))
    };

    let flags = web_sys::FileSystemFlags::default();
    let _ = directory_entry.get_file_with_path_and_options_and_callback_and_callback(
      Some(path),
      &flags,
      success.as_ref().unchecked_ref(),
      error.as_ref().unchecked_ref()
    );

    state.borrow_mut().callbacks = Some((success, error));

    FileSystemDirectoryEntry_getFile_future { inner: state }
  }
}

impl Future for FileSystemDirectoryEntry_getFile_future {
  type Output = Result<SuccessType, ErrorType>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
    let mut inner = self.inner.borrow_mut();

    if let Some(val) = inner.result.take() {
      return Poll::Ready(val);
    }

    inner.task = Some(cx.waker().clone());
    Poll::Pending
  }
}
