use crate::prelude::*;

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug)]
pub struct WasmError {
  pub code: u32,
  pub arg0: String,
  pub arg1: String,
  pub arg2: String,
  pub arg3: String,
}

impl WasmError {
  fn new(code: u32, arg0: &str, arg1: &str, arg2: &str, arg3: &str) -> Self {
    Self {
      code,
      arg0: arg0.to_string(),
      arg1: arg1.to_string(),
      arg2: arg2.to_string(),
      arg3: arg3.to_string(),
    }
  }

  pub fn dynamic_cast_error(location: &str, from_type: &str, to_type: &str) -> Self {
    Self::new(
      0x00000000_u32,
      location,
      from_type,
      to_type,
      "",
    )
  }

  pub fn unknown_file_entry(location: &str) -> Self {
    Self::new(
      0x00000001_u32,
      location,
      "",
      "",
      "",
    )
  }

  pub fn empty_file_list(location: &str) -> Self {
    Self::new(
      0x00000002_u32,
      location,
      "",
      "",
      "",
    )
  }

  pub fn missing_file_system(location: &str) -> Self {
    Self::new(
      0x00000003_u32,
      location,
      "",
      "",
      "",
    )
  }

  pub fn fail_to_write(location: &str, upstream_error: &str) -> Self {
    Self::new(
      0x00000004_u32,
      location,
      upstream_error,
      "",
      "",
    )
  }

  pub fn fail_to_get_file_entry(location: &str, upstream_error: &str) -> Self {
    Self::new(
      0x00000005_u32,
      location,
      upstream_error,
      "",
      "",
    )
  }

  pub fn fail_to_get_file(location: &str, upstream_error: &str) -> Self {
    Self::new(
      0x00000006_u32,
      location,
      upstream_error,
      "",
      "",
    )
  }

  pub fn fail_to_read(location: &str, upstream_error: &str) -> Self {
    Self::new(
      0x00000007_u32,
      location,
      upstream_error,
      "",
      "",
    )
  }

  pub fn read_stream_cancelled(location: &str) -> Self {
    Self::new(
      0x00000008_u32,
      location,
      "",
      "",
      "",
    )
  }

  pub fn fail_to_compress(location: &str, upstream_error: &str) -> Self {
    Self::new(
      0x00000009_u32,
      location,
      upstream_error,
      "",
      "",
    )
  }

  pub fn fail_to_close_writer(location: &str, upstream_error: &str) -> Self {
    Self::new(
      0x0000000A_u32,
      location,
      upstream_error,
      "",
      "",
    )
  }

  pub fn too_many_files(location: &str) -> Self {
    Self::new(
      0x0000000B_u32,
      location,
      "",
      "",
      "",
    )
  }

  pub fn can_not_recover_directory(location: &str, path: &str) -> Self {
    Self::new(
      0x0000000C_u32,
      location,
      path,
      "",
      "",
    )
  }

  pub fn invalid_compression_level(location: &str, compression_level: &str) -> Self {
    Self::new(
      0x0000000D_u32,
      location,
      compression_level,
      "",
      "",
    )
  }

  pub fn fail_to_invoke_callback(location: &str, callback_name: &str, upstream_error: &str) -> Self {
    Self::new(
      0x0000000E_u32,
      location,
      callback_name,
      upstream_error,
      "",
    )
  }

  pub fn fail_to_create_writer(location: &str, upstream_error: &str) -> Self {
    Self::new(
      0x0000000F_u32,
      location,
      upstream_error,
      "",
      "",
    )
  }

  pub fn can_not_transform_directory(location: &str, path: &str) -> Self {
    Self::new(
      0x00000010_u32,
      location,
      path,
      "",
      "",
    )
  }
}
