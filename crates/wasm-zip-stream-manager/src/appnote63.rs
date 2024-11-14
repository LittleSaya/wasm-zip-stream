//! Designed against APPNOTE 6.3
//!
//! The type `FileHeader` is used to generate both the "local file header" and the "central directory header".

use web_sys::wasm_bindgen;
use wasm_bindgen::prelude::*;

use crate::constant::{COMPRESSION_METHOD, COMPRESSION_METHOD_DIR, GENERAL_PURPOSE_BIG_FLAG, GENERAL_PURPOSE_BIG_FLAG_DIR, VERSION_MADE_BY, VERSION_NEEDED_TO_EXTRACT};

#[wasm_bindgen]
pub struct FileHeader {
  is_dir                : bool,
  lfh_pos               : u64,
  file_name_length      : u16,
  file_name             : Vec<u8>,
  crc_32                : u32,
  compressed_size_u64   : u64,
  uncompressed_size_u64 : u64,
}

impl FileHeader {
  /// # Parameters
  ///
  /// * `file_name` - File name could be a relative path, which MUST not contain a driver or device
  ///                 letter, or a leading slash. All slashes MUST be '/'. (Ref 4.4.17)
  /// * `lfh_pos`   - local file header position
  /// * `is_dir`    - `true` if this header represents a directory, `false` otherwise
  pub fn new(file_name: String, lfh_pos: u64, is_dir: bool) -> Self {
    let file_name: Vec<u8> = file_name.into();

    Self {
      is_dir,
      lfh_pos,
      file_name_length: file_name.len() as u16,
      file_name,

      // if this header represents a file, these 3 values will be filled after finishing compressing this file
      crc_32: 0_u32,
      compressed_size_u64: 0_u64,
      uncompressed_size_u64: 0_u64,
    }
  }

  /// Clear the buffer, then write the "local file header" into the buffer.
  pub fn write_into_as_lfh(&self, buffer: &mut Vec<u8>) {
    if self.is_dir {
      buffer.clear();

      buffer.extend_from_slice(&0x04034b50_u32.to_le_bytes()); // signature
      buffer.extend_from_slice(&VERSION_NEEDED_TO_EXTRACT.to_le_bytes());
      buffer.extend_from_slice(&GENERAL_PURPOSE_BIG_FLAG_DIR.to_le_bytes());
      buffer.extend_from_slice(&COMPRESSION_METHOD_DIR.to_le_bytes());
      buffer.extend_from_slice(&0_u16.to_le_bytes()); // last mod file time
      buffer.extend_from_slice(&0_u16.to_le_bytes()); // last mod file date
      buffer.extend_from_slice(&0_u32.to_le_bytes()); // crc32
      buffer.extend_from_slice(&0_u32.to_le_bytes()); // compressed size
      buffer.extend_from_slice(&0_u32.to_le_bytes()); // uncompressed size
      buffer.extend_from_slice(&self.file_name_length.to_le_bytes());
      buffer.extend_from_slice(&0_u16.to_le_bytes());
      buffer.extend_from_slice(&self.file_name);
    } else {
      const EXTRA_FIELD_LENGTH: u16 = 20_u16;
      let mut extra_field = [0_u8; EXTRA_FIELD_LENGTH as usize];
      extra_field[ 0.. 2].copy_from_slice(&0x0001_u16.to_le_bytes()); // 2 bytes    Tag for this "extra" block type
      extra_field[ 2.. 4].copy_from_slice(&16_u16.to_le_bytes());     // 2 bytes    Size of this "extra" block
      extra_field[ 4..12].copy_from_slice(&0_u64.to_le_bytes());      // 8 bytes    Original uncompressed file size
      extra_field[12..20].copy_from_slice(&0_u64.to_le_bytes());      // 8 bytes    Size of compressed data

      buffer.clear();

      buffer.extend_from_slice(&0x04034b50_u32.to_le_bytes()); // signature
      buffer.extend_from_slice(&VERSION_NEEDED_TO_EXTRACT.to_le_bytes());
      buffer.extend_from_slice(&GENERAL_PURPOSE_BIG_FLAG.to_le_bytes());
      buffer.extend_from_slice(&COMPRESSION_METHOD.to_le_bytes());
      buffer.extend_from_slice(&0_u16.to_le_bytes()); // last mod file time
      buffer.extend_from_slice(&0_u16.to_le_bytes()); // last mod file date
      buffer.extend_from_slice(&0_u32.to_le_bytes()); // crc32
      buffer.extend_from_slice(&0xFFFFFFFF_u32.to_le_bytes()); // compressed size
      buffer.extend_from_slice(&0xFFFFFFFF_u32.to_le_bytes()); // uncompressed size
      buffer.extend_from_slice(&self.file_name_length.to_le_bytes());
      buffer.extend_from_slice(&EXTRA_FIELD_LENGTH.to_le_bytes());
      buffer.extend_from_slice(&self.file_name);
      buffer.extend_from_slice(&extra_field);
    }
  }

  /// Without clearing the buffer, write the "central directory header" into the buffer.
  pub fn write_into_as_cdh(&self, buffer: &mut Vec<u8>) {
    if self.is_dir {
      const EXTRA_FIELD_LENGTH: u16 = 16_u16;
      let mut extra_field = [0_u8; EXTRA_FIELD_LENGTH as usize];
      extra_field[ 0.. 2].copy_from_slice(&0x0001_u16.to_le_bytes());                 // 2 bytes    Tag for this "extra" block type
      extra_field[ 2.. 4].copy_from_slice(&12_u16.to_le_bytes());                     // 2 bytes    Size of this "extra" block
      extra_field[ 4..12].copy_from_slice(&self.lfh_pos.to_le_bytes());               // 8 bytes    Offset of local header record
      extra_field[12..16].copy_from_slice(&0_u32.to_le_bytes());                      // 4 bytes    Number of the disk on which this file starts

      buffer.extend_from_slice(&0x02014b50_u32.to_le_bytes()); // signature
      buffer.extend_from_slice(&VERSION_MADE_BY.to_le_bytes());
      buffer.extend_from_slice(&VERSION_NEEDED_TO_EXTRACT.to_le_bytes());
      buffer.extend_from_slice(&GENERAL_PURPOSE_BIG_FLAG_DIR.to_le_bytes());
      buffer.extend_from_slice(&COMPRESSION_METHOD_DIR.to_le_bytes());
      buffer.extend_from_slice(&0_u16.to_le_bytes()); // last mod file time
      buffer.extend_from_slice(&0_u16.to_le_bytes()); // last mod file date
      buffer.extend_from_slice(&0_u32.to_le_bytes()); // crc32
      buffer.extend_from_slice(&0_u32.to_le_bytes()); // compressed size
      buffer.extend_from_slice(&0_u32.to_le_bytes()); // uncompressed size
      buffer.extend_from_slice(&self.file_name_length.to_le_bytes());
      buffer.extend_from_slice(&EXTRA_FIELD_LENGTH.to_le_bytes()); // extra field length
      buffer.extend_from_slice(&0_u16.to_le_bytes()); // file comment length
      buffer.extend_from_slice(&0xFFFF_u16.to_le_bytes()); // disk number start
      buffer.extend_from_slice(&0_u16.to_le_bytes()); // internal file attributes
      buffer.extend_from_slice(&0_u32.to_le_bytes()); // external file attributes
      buffer.extend_from_slice(&0xFFFFFFFF_u32.to_le_bytes()); // relative offset of local header
      buffer.extend_from_slice(&self.file_name);
      buffer.extend_from_slice(&extra_field);
    } else {
      const EXTRA_FIELD_LENGTH: u16 = 32_u16;
      let mut extra_field = [0_u8; EXTRA_FIELD_LENGTH as usize];
      extra_field[ 0.. 2].copy_from_slice(&0x0001_u16.to_le_bytes());                 // 2 bytes    Tag for this "extra" block type
      extra_field[ 2.. 4].copy_from_slice(&28_u16.to_le_bytes());                     // 2 bytes    Size of this "extra" block
      extra_field[ 4..12].copy_from_slice(&self.uncompressed_size_u64.to_le_bytes()); // 8 bytes    Original uncompressed file size
      extra_field[12..20].copy_from_slice(&self.compressed_size_u64.to_le_bytes());   // 8 bytes    Size of compressed data
      extra_field[20..28].copy_from_slice(&self.lfh_pos.to_le_bytes());               // 8 bytes    Offset of local header record
      extra_field[28..32].copy_from_slice(&0_u32.to_le_bytes());                      // 4 bytes    Number of the disk on which this file starts

      buffer.extend_from_slice(&0x02014b50_u32.to_le_bytes()); // signature
      buffer.extend_from_slice(&VERSION_MADE_BY.to_le_bytes());
      buffer.extend_from_slice(&VERSION_NEEDED_TO_EXTRACT.to_le_bytes());
      buffer.extend_from_slice(&GENERAL_PURPOSE_BIG_FLAG.to_le_bytes());
      buffer.extend_from_slice(&COMPRESSION_METHOD.to_le_bytes());
      buffer.extend_from_slice(&0_u16.to_le_bytes()); // last mod file time
      buffer.extend_from_slice(&0_u16.to_le_bytes()); // last mod file date
      buffer.extend_from_slice(&self.crc_32.to_le_bytes());
      buffer.extend_from_slice(&0xFFFFFFFF_u32.to_le_bytes()); // compressed size
      buffer.extend_from_slice(&0xFFFFFFFF_u32.to_le_bytes()); // uncompressed size
      buffer.extend_from_slice(&self.file_name_length.to_le_bytes());
      buffer.extend_from_slice(&EXTRA_FIELD_LENGTH.to_le_bytes());
      buffer.extend_from_slice(&0_u16.to_le_bytes()); // file comment length
      buffer.extend_from_slice(&0xFFFF_u16.to_le_bytes()); // disk number start
      buffer.extend_from_slice(&0_u16.to_le_bytes()); // internal file attributes
      buffer.extend_from_slice(&0_u32.to_le_bytes()); // external file attributes
      buffer.extend_from_slice(&0xFFFFFFFF_u32.to_le_bytes()); // relative offset of local header
      buffer.extend_from_slice(&self.file_name);
      buffer.extend_from_slice(&extra_field);
    }
  }

  pub fn set_crc_32(&mut self, crc32: u32) {
    self.crc_32 = crc32;
  }

  pub fn set_compressed_size_u64(&mut self, size: u64) {
    self.compressed_size_u64 = size;
  }

  pub fn set_uncompressed_size_u64(&mut self, size: u64) {
    self.uncompressed_size_u64 = size;
  }
}
