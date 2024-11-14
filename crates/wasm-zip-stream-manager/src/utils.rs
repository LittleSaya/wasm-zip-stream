use crate::{prelude::*, WasmError};

pub fn set_panic_hook() {
  // When the `console_error_panic_hook` feature is enabled, we can call the
  // `set_panic_hook` function at least once during initialization, and then
  // we will get better error messages if our code ever panics.
  //
  // For more details see
  // https://github.com/rustwasm/console_error_panic_hook#readme
  console_error_panic_hook::set_once();
}

pub async fn await_promise(promise: js_sys::Promise) -> Result<JsValue, JsValue> {
  wasm_bindgen_futures::JsFuture::from(promise).await
}

pub async fn write(writer: &web_sys::WritableStreamDefaultWriter, view: &js_sys::Uint8Array) -> Result<(), WasmError> {
  if let Err(e) = await_promise(writer.write_with_chunk(view)).await {
    return Err(WasmError::fail_to_write(type_name(&write), &format!("{:?}", e)));
  }
  Ok(())
}

#[wasm_bindgen]
extern "C" {
  type ReadResultJS;

  #[wasm_bindgen(method, getter)]
  fn value(this: &ReadResultJS) -> JsValue;

  #[wasm_bindgen(method, getter)]
  fn done(this: &ReadResultJS) -> JsValue;
}

struct ReadResultSingle {
  value: js_sys::Uint8Array,
  done: bool,
}

pub struct ReadResult {
  pub new_buffer: js_sys::ArrayBuffer,
  pub view: js_sys::Uint8Array,
  pub done: bool,
}

pub async fn byob_read(old_array_buffer: &js_sys::ArrayBuffer, reader: &web_sys::ReadableStreamByobReader) -> Result<ReadResult, WasmError> {
  #[allow(non_snake_case)]
  let LOCATION = type_name(&byob_read);

  const MINIMAL_CHUNK_SIZE: u32 = 64 * 1024; // 64 KiB

  let buffer_size = old_array_buffer.byte_length();
  let mut bytes_read = 0_u32;

  let first_read_result = match await_promise(reader.read_with_array_buffer_view(&js_sys::Uint8Array::new(old_array_buffer))).await {
    Ok(read_result) => {
      let read_result = read_result.unchecked_into::<ReadResultJS>();

      let value = read_result.value();
      if value.eq(&JsValue::UNDEFINED) {
        return Err(WasmError::read_stream_cancelled(LOCATION));
      }
      let Ok(value) = value.dyn_into::<js_sys::Uint8Array>() else {
        return Err(WasmError::dynamic_cast_error(LOCATION, "JsValue", "Uint8Array"));
      };

      bytes_read += value.byte_length();

      let done = read_result.done().eq(&JsValue::TRUE);

      ReadResultSingle { value, done }
    },
    Err(e) => return Err(WasmError::fail_to_read(LOCATION, &format!("{:?}", e))),
  };

  if first_read_result.done {
    let new_buffer = first_read_result.value.buffer();
    let view = js_sys::Uint8Array::new_with_byte_offset_and_length(&new_buffer, 0, bytes_read);
    return Ok(ReadResult { new_buffer, view, done: true });
  }

  let mut new_array_buffer = first_read_result.value.buffer();

  loop {
    if buffer_size - bytes_read < MINIMAL_CHUNK_SIZE {
      let view = js_sys::Uint8Array::new_with_byte_offset_and_length(&new_array_buffer, 0, bytes_read);
      return Ok(ReadResult { new_buffer: new_array_buffer, view, done: false });
    }

    let read_result = match await_promise(reader.read_with_array_buffer_view(
      &js_sys::Uint8Array::new_with_byte_offset_and_length(
        &new_array_buffer,
        bytes_read,
        buffer_size - bytes_read,
      )
    )).await {
      Ok(read_result) => {
        let read_result = read_result.unchecked_into::<ReadResultJS>();

        let value = read_result.value();
        if value.eq(&JsValue::UNDEFINED) {
          return Err(WasmError::read_stream_cancelled(LOCATION));
        }
        let Ok(value) = value.dyn_into::<js_sys::Uint8Array>() else {
          return Err(WasmError::dynamic_cast_error(LOCATION, "JsValue", "Uint8Array"));
        };

        bytes_read += value.byte_length();

        let done = read_result.done().eq(&JsValue::TRUE);

        ReadResultSingle { value, done }
      },
      Err(e) => return Err(WasmError::fail_to_read(LOCATION, &format!("{:?}", e))),
    };

    new_array_buffer = read_result.value.buffer();
    if read_result.done {
      let view = js_sys::Uint8Array::new_with_byte_offset_and_length(&new_array_buffer, 0, bytes_read);
      return Ok(ReadResult { new_buffer: new_array_buffer, view, done: true });
    }
  }
}

pub fn type_name<T: ?Sized>(_val: &T) -> &'static str {
  std::any::type_name::<T>()
}
