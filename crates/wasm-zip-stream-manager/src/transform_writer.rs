//! NOT IMPLEMENTED

use std::io::Write;

use crate::{prelude::*, utils};
use crate::wasm_error::WasmError;

pub struct TransformWriter<'a> {
  writer: &'a web_sys::WritableStreamDefaultWriter,
  threshold: usize,
  transform_buffer: Vec<u8>,
  result_buffer: Vec<u8>,
  bypass: bool,
}

impl<'a> TransformWriter<'a> {
  pub fn new(
    writer: &'a web_sys::WritableStreamDefaultWriter,
    threshold: usize,
    transform_buffer_size: usize,
    result_buffer_size: usize,
    bypass: bool
  ) -> Self {
    Self {
      writer,
      threshold,
      transform_buffer: Vec::with_capacity(transform_buffer_size),
      result_buffer: Vec::with_capacity(result_buffer_size),
      bypass,
    }
  }

  pub async fn dump(&mut self, use_threshold: bool) -> Result<(), WasmError> {
    if (use_threshold && self.result_buffer.len() >= self.threshold) || (!use_threshold && self.result_buffer.len() > 0) {
      let array_buffer_write = js_sys::ArrayBuffer::new(self.result_buffer.len() as u32);
      let view_write = js_sys::Uint8Array::new(&array_buffer_write);
      view_write.copy_from(&self.result_buffer);
      utils::await_promise(self.writer.ready()).await.unwrap();
      utils::write(self.writer, &view_write).await?;
      self.result_buffer.clear();
    }
    Ok(())
  }
}

impl<'a> Write for TransformWriter<'a> {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    if self.bypass {
      self.result_buffer.extend_from_slice(buf);
    } else {
      for byte in buf {
        // full, apply a full transformation and push transformed bytes into result buffer
        if self.transform_buffer.len() == self.transform_buffer.capacity() {
          for i in 0..self.transform_buffer.len() {
            self.transform_buffer[i] ^= 0xFF_u8;
          }
          self.result_buffer.extend_from_slice(&self.transform_buffer);
          self.transform_buffer.clear();
        }

        self.transform_buffer.push(*byte);
      }
    }
    Ok(buf.len())
  }

  fn flush(&mut self) -> std::io::Result<()> {
    if self.bypass {
      // no bytes left in transform buffer
    } else {
      if self.transform_buffer.len() == self.transform_buffer.capacity() {
        // full, apply a full transformation and push transformed bytes into result buffer
        for i in 0..self.transform_buffer.len() {
          self.transform_buffer[i] ^= 0xFF_u8;
        }
        self.result_buffer.extend_from_slice(&self.transform_buffer);
        self.transform_buffer.clear();
      } else {
        // not full, apply a trailing transformation and push transformed bytes into result buffer
        for i in 0..self.transform_buffer.len() {
          self.transform_buffer[i] ^= 0xFF_u8;
        }
        self.result_buffer.extend_from_slice(&self.transform_buffer);
        self.transform_buffer.clear();
      }
    }
    Ok(())
  }
}
