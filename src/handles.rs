//! The type `Handles` is the main entrance where users operate on this library.

use std::io::Write;
use std::rc::Rc;

use crc32fast::Hasher;
use flate2::{write::DeflateEncoder, Compression};

use crate::constant::{VERSION_MADE_BY, VERSION_NEEDED_TO_EXTRACT};
use crate::recover_writer::RecoverWriter;
use crate::transform_writer::TransformWriter;
use crate::utils::ReadResult;
use crate::{appnote63, js_futures, prelude::*, utils};
use crate::context::{Context, FilePath};
use crate::wasm_error::WasmError;

#[wasm_bindgen]
pub struct Handles {
  context: Rc<Context>,

  /// unit: ms
  speed_report_interval: f64,

  create_writer: js_sys::Function,

  scan_progress: Option<js_sys::Function>,
  compress_progress: Option<js_sys::Function>,
  average_speed: Option<js_sys::Function>,
  current_speed: Option<js_sys::Function>,
  current_file: Option<js_sys::Function>,
}

impl Handles {
  pub fn new(context: &Rc<Context>, create_writer: js_sys::Function) -> Self {
    Self {
      context: Rc::clone(context),
      speed_report_interval: 5000_f64,
      create_writer,
      scan_progress: None,
      compress_progress: None,
      average_speed: None,
      current_speed: None,
      current_file: None,
    }
  }

  async fn scan_internal(&self, entries: js_sys::Array) -> Result<JsValue, WasmError> {
    #[allow(non_snake_case, unused_variables)]
    let LOCATION = utils::type_name(&Self::scan_internal);

    let array_length = entries.length();
    let mut temp_vector = Vec::<web_sys::FileSystemEntry>::with_capacity(array_length as usize);
    for i in 0..array_length {
      temp_vector.push(entries.get(i).unchecked_into::<web_sys::FileSystemEntry>());
    }

    let entries = temp_vector;
    for entry in entries {
      let full_path = entry.full_path();

      if entry.is_file() {
        self.context.scan_stage.file_path_list.borrow_mut().push(FilePath { path: full_path, is_dir: false });

        self.report_scan_progress(self.context.scan_stage.file_path_list.borrow().len())?;
      }
      else if entry.is_directory() {
        self.context.scan_stage.file_path_list.borrow_mut().push(FilePath { path: full_path, is_dir: true });

        self.report_scan_progress(self.context.scan_stage.file_path_list.borrow().len())?;

        let directory_entry = entry.unchecked_into::<web_sys::FileSystemDirectoryEntry>();
        let reader = directory_entry.create_reader();

        let mut array = js_sys::Array::new();
        loop {
          let cloned_reader = reader.clone();
          let partial: js_sys::Array = js_futures::FileSystemDirectoryReader_readEntries_future::from(cloned_reader).await.unwrap();
          if partial.length() > 0 {
            array = array.concat(&partial);
          } else {
            break;
          }
        }

        if let Err(e) = Box::pin(self.scan_internal(array)).await {
          return Err(e);
        }
      }
      else {
        return Err(WasmError::unknown_file_entry(utils::type_name(&Handles::scan)));
      }
    }

    Ok(JsValue::UNDEFINED)
  }

  async fn compress_internal(&self, mut output_file_name: String, compression_level: u32, transform_script: Option<String>) -> Result<JsValue, WasmError> {
    #[allow(non_snake_case)]
    let LOCATION = utils::type_name(&Self::compress_internal);

    if compression_level > 9 {
      return Err(WasmError::invalid_compression_level(LOCATION, &format!("{}", compression_level)));
    }

    if self.context.scan_stage.file_path_list.borrow().len() == 0 {
      return Err(WasmError::empty_file_list(LOCATION));
    }
    let file_path_list = self.context.scan_stage.file_path_list.borrow();

    let file_system = self.context.scan_stage.file_system.borrow();
    let Some(file_system) = file_system.as_ref() else {
      return Err(WasmError::missing_file_system(LOCATION));
    };

    if !output_file_name.ends_with(".zip") {
      output_file_name += ".zip";
    }
    let writer = match self.create_writer.call1(&JsValue::NULL, &JsValue::from_str(&output_file_name)) {
      Ok(w) => w,
      Err(e) => return Err(WasmError::fail_to_create_writer(LOCATION, &format!("{:?}", e))),
    };
    let Ok(writer) = writer.dyn_into::<web_sys::WritableStreamDefaultWriter>() else {
      return Err(WasmError::dynamic_cast_error(LOCATION, "JsValue", "WritableStreamDefaultWriter"));
    };

    let mut bytes_written = 0_u64;

    let mut file_headers = Vec::with_capacity(file_path_list.len());

    let mut buffer_header = Vec::<u8>::with_capacity(64 * 1024);
    let mut buffer_read = vec![0_u8; 16 * 1024 * 1024];
    let mut array_buffer_read = js_sys::ArrayBuffer::new(16 * 1024 * 1024);

    const TRANSFORM_BUFFER_WRITE_THRESHOLD: usize = 16 * 1024 * 1024;
    let mut transform_writer = TransformWriter::new(
      &writer,
      TRANSFORM_BUFFER_WRITE_THRESHOLD,
      8,
      16 * 1024 * 1024,
      transform_script.is_none(),
    );

    let mut deflate_encoder = DeflateEncoder::new(Vec::<u8>::with_capacity(16 * 1024 * 1024), Compression::new(compression_level));
    let mut crc32_hasher = Hasher::new();

    self.report_compress_progress(file_headers.len(), file_headers.capacity())?;

    let speed_report_start_time = self.context.performance.now();
    let mut speed_report_last_time = speed_report_start_time;
    let mut speed_report_current_time;
    let mut speed_report_delta_time;
    let mut speed_report_last_total_bytes = 0;
    let mut speed_report_current_total_bytes;
    let mut speed_report_delta_total_bytes;

    for FilePath { path, is_dir } in file_path_list.iter() { // start of file loop
      self.report_current_file(path)?;

      // create file header
      let mut zip_path = path.trim_start_matches('/').to_owned();
      if *is_dir {
        zip_path.push('/');
      }
      let mut file_header = appnote63::FileHeader::new(zip_path, bytes_written, *is_dir);

      // write local file header
      file_header.write_into_as_lfh(&mut buffer_header);
      transform_writer.write(&buffer_header).unwrap();
      bytes_written += buffer_header.len() as u64;

      transform_writer.dump(true).await?;

      // speed measurement
      speed_report_current_time = self.context.performance.now();
      speed_report_current_total_bytes = bytes_written;
      speed_report_delta_time = speed_report_current_time - speed_report_last_time;
      speed_report_delta_total_bytes = speed_report_current_total_bytes - speed_report_last_total_bytes;
      if speed_report_delta_time >= self.speed_report_interval {
        self.report_average_speed(speed_report_current_total_bytes, speed_report_current_time - speed_report_start_time)?;
        self.report_current_speed(speed_report_delta_total_bytes, speed_report_delta_time)?;
        speed_report_last_time = speed_report_current_time;
        speed_report_last_total_bytes = speed_report_current_total_bytes;
      }

      if *is_dir {
        file_headers.push(file_header);
        continue;
      }

      // get FileSystemFileEntry
      let file_entry = match js_futures::FileSystemDirectoryEntry_getFile_future::from(file_system.root(), path).await {
        Ok(value) => value,
        Err(e) => return Err(WasmError::fail_to_get_file_entry(LOCATION, &format!("{:?}", e))),
      };

      // get File
      let file = match js_futures::FileSystemFileEntry_file_future::from(file_entry).await {
        Ok(file) => file,
        Err(e) => return Err(WasmError::fail_to_get_file(LOCATION, &format!("{:?}", e))),
      };

      // cast File to Blob
      let Ok(blob) = file.dyn_into::<web_sys::Blob>() else {
        return Err(WasmError::dynamic_cast_error(LOCATION, "File", "Blob"));
      };

      // get BYOB reader
      let stream = blob.stream();
      let get_reader_option = web_sys::ReadableStreamGetReaderOptions::new();
      get_reader_option.set_mode(web_sys::ReadableStreamReaderMode::Byob);
      let Ok(reader) = stream.get_reader_with_options(&get_reader_option).dyn_into::<web_sys::ReadableStreamByobReader>() else {
        return Err(WasmError::dynamic_cast_error(LOCATION, "Object", "ReadableStreamByobReader"));
      };

      let mut uncompressed_size = 0_u64;
      let mut compressed_size = 0_u64;

      // the read-compress-write loop
      loop { // start of compress loop
        // js reader -> js buffer
        let ReadResult { new_buffer: array_buffer_new_read, view: view_read, done } = utils::byob_read(&array_buffer_read, &reader).await?;

        let bytes_read = view_read.byte_length();
        uncompressed_size += bytes_read as u64;

        array_buffer_read = array_buffer_new_read; // replace old detached ArrayBuffer immediately

        // if indeed read some bytes
        if bytes_read > 0 {
          // js buffer -> wasm buffer slice
          let wasm_slice = &mut buffer_read[0..bytes_read as usize];
          view_read.copy_to(wasm_slice);

          // wasm buffer slice -> crc hasher
          crc32_hasher.update(wasm_slice);

          // wasm buffer slice -> deflate encoder -> inner buffer
          deflate_encoder.get_mut().clear();
          if let Err(e) = deflate_encoder.write_all(wasm_slice) {
            return Err(WasmError::fail_to_compress(LOCATION, &format!("{:?}", e)));
          }

          let bytes_output = deflate_encoder.get_ref().len();

          // if indeed get some compressed bytes
          if bytes_output > 0 {
            compressed_size += bytes_output as u64;

            // inner buffer -> transform writer
            transform_writer.write(deflate_encoder.get_ref()).unwrap();

            transform_writer.dump(true).await?;

            // speed measurement
            speed_report_current_time = self.context.performance.now();
            speed_report_current_total_bytes = bytes_written + compressed_size;
            speed_report_delta_time = speed_report_current_time - speed_report_last_time;
            speed_report_delta_total_bytes = speed_report_current_total_bytes - speed_report_last_total_bytes;
            if speed_report_delta_time >= self.speed_report_interval {
              self.report_average_speed(speed_report_current_total_bytes, speed_report_current_time - speed_report_start_time)?;
              self.report_current_speed(speed_report_delta_total_bytes, speed_report_delta_time)?;
              speed_report_last_time = speed_report_current_time;
              speed_report_last_total_bytes = speed_report_current_total_bytes;
            }
          }
        }

        if done {
          // deflate encoder -> inner buffer
          deflate_encoder.get_mut().clear();
          let inner_buffer = match deflate_encoder.finish() {
            Ok(inner_buffer) => inner_buffer,
            Err(e) => return Err(WasmError::fail_to_compress(LOCATION, &format!("{:?}", e))),
          };

          // if indeed we get some final extra compressed bytes to write
          let bytes_output = inner_buffer.len();
          if bytes_output > 0 {
            compressed_size += bytes_output as u64;

            // inner buffer -> transform writer
            transform_writer.write(&inner_buffer).unwrap();

            transform_writer.dump(true).await?;

            // speed measurement
            speed_report_current_time = self.context.performance.now();
            speed_report_current_total_bytes = bytes_written + compressed_size;
            speed_report_delta_time = speed_report_current_time - speed_report_last_time;
            speed_report_delta_total_bytes = speed_report_current_total_bytes - speed_report_last_total_bytes;
            if speed_report_delta_time >= self.speed_report_interval {
              self.report_average_speed(speed_report_current_total_bytes, speed_report_current_time - speed_report_start_time)?;
              self.report_current_speed(speed_report_delta_total_bytes, speed_report_delta_time)?;
              speed_report_last_time = speed_report_current_time;
              speed_report_last_total_bytes = speed_report_current_total_bytes;
            }
          }

          // recreate the deflate encoder
          deflate_encoder = DeflateEncoder::new(inner_buffer, Compression::new(compression_level));

          break;
        }
      } // end of compress loop

      // finalize crc32 and recreate the hasher
      let crc32 = crc32_hasher.finalize();
      crc32_hasher = Hasher::new();

      bytes_written += compressed_size;

      // write data descriptor

      const DATA_DESCRIPTOR_SIZE: u32 = 4 + 4 + 8 + 8;

      let mut data_descriptor_buffer = Vec::<u8>::with_capacity(DATA_DESCRIPTOR_SIZE as usize);
      data_descriptor_buffer.extend_from_slice(&0x08074b50_u32.to_le_bytes());
      data_descriptor_buffer.extend_from_slice(&crc32.to_le_bytes());
      data_descriptor_buffer.extend_from_slice(&compressed_size.to_le_bytes());
      data_descriptor_buffer.extend_from_slice(&uncompressed_size.to_le_bytes());

      // wasm buffer -> transform writer
      transform_writer.write(&data_descriptor_buffer).unwrap();

      transform_writer.dump(true).await?;

      bytes_written += DATA_DESCRIPTOR_SIZE as u64;

      // speed measurement
      speed_report_current_time = self.context.performance.now();
      speed_report_current_total_bytes = bytes_written;
      speed_report_delta_time = speed_report_current_time - speed_report_last_time;
      speed_report_delta_total_bytes = speed_report_current_total_bytes - speed_report_last_total_bytes;
      if speed_report_delta_time >= self.speed_report_interval {
        self.report_average_speed(speed_report_current_total_bytes, speed_report_current_time - speed_report_start_time)?;
        self.report_current_speed(speed_report_delta_total_bytes, speed_report_delta_time)?;
        speed_report_last_time = speed_report_current_time;
        speed_report_last_total_bytes = speed_report_current_total_bytes;
      }

      // populate missing part in file header
      file_header.set_crc_32(crc32);
      file_header.set_compressed_size_u64(compressed_size);
      file_header.set_uncompressed_size_u64(uncompressed_size);

      file_headers.push(file_header);

      self.report_compress_progress(file_headers.len(), file_headers.capacity())?;
    } // end of file loop

    // write tail data

    let start_of_central_directory = bytes_written;

    let mut tail_buffer = Vec::<u8>::with_capacity(32 * 1024 * 1024 as usize); // 32 MiB, should be able to hold at least 100k file/directory entries

    // central directory headers
    let number_of_file_headers = file_headers.len() as u64;
    for file_header in file_headers {
      file_header.write_into_as_cdh(&mut tail_buffer);
    }

    // the size of all central directory headers
    let size_of_central_directory = tail_buffer.len() as u64;

    let relative_offset_of_zip64_end_of_central_directory_record = start_of_central_directory + size_of_central_directory;

    // zip64 end of central directory record
    tail_buffer.extend_from_slice(&0x06064b50_u32.to_le_bytes()); // zip64 end of central dir signature
    tail_buffer.extend_from_slice(&(2_u64 + 2_u64 + 4_u64 + 4_u64 + 8_u64 + 8_u64 + 8_u64 + 8_u64).to_le_bytes());
    tail_buffer.extend_from_slice(&VERSION_MADE_BY.to_le_bytes());
    tail_buffer.extend_from_slice(&VERSION_NEEDED_TO_EXTRACT.to_le_bytes());
    tail_buffer.extend_from_slice(&0_u32.to_le_bytes()); // splitting is not supported
    tail_buffer.extend_from_slice(&0_u32.to_le_bytes()); // splitting is not supported
    tail_buffer.extend_from_slice(&number_of_file_headers.to_le_bytes());
    tail_buffer.extend_from_slice(&number_of_file_headers.to_le_bytes());
    tail_buffer.extend_from_slice(&size_of_central_directory.to_le_bytes());
    tail_buffer.extend_from_slice(&start_of_central_directory.to_le_bytes());

    // zip64 end of central directory locator
    tail_buffer.extend_from_slice(&0x07064b50_u32.to_le_bytes()); // zip64 end of central dir locator signature
    tail_buffer.extend_from_slice(&0_u32.to_le_bytes()); // splitting is not supported
    tail_buffer.extend_from_slice(&relative_offset_of_zip64_end_of_central_directory_record.to_le_bytes());
    tail_buffer.extend_from_slice(&1_u32.to_le_bytes()); // splitting is not supported

    // End of central directory record
    tail_buffer.extend_from_slice(&0x06054b50_u32.to_le_bytes());
    tail_buffer.extend_from_slice(&0xFFFF_u16.to_le_bytes()); // zip64, number of this disk
    tail_buffer.extend_from_slice(&0xFFFF_u16.to_le_bytes()); // zip64, number of the disk with the start of the central directory
    tail_buffer.extend_from_slice(&0xFFFF_u16.to_le_bytes()); // zip64, total number of entries in the central directory on this disk
    tail_buffer.extend_from_slice(&0xFFFF_u16.to_le_bytes()); // zip64, total number of entries in the central directory
    tail_buffer.extend_from_slice(&0xFFFFFFFF_u32.to_le_bytes()); // zip64, size of the central directory
    tail_buffer.extend_from_slice(&0xFFFFFFFF_u32.to_le_bytes()); // zip64, offset of start of central directory with respect to the starting disk number
    tail_buffer.extend_from_slice(&0_u16.to_le_bytes()); // zip file comment length
    // central_directory_buffer.extend_from_slice(&[0_u8; 0]); // zip file comment (no comment)

    // tail buffer -> transform buffer
    transform_writer.write(&tail_buffer).unwrap();

    // no more data
    transform_writer.flush().unwrap();

    transform_writer.dump(false).await?;

    // final speed measurement
    speed_report_current_time = self.context.performance.now();
    speed_report_delta_time = speed_report_current_time - speed_report_last_time;
    self.report_average_speed(bytes_written, speed_report_current_time - speed_report_start_time)?;
    self.report_current_speed(bytes_written - speed_report_last_total_bytes, speed_report_delta_time)?;

    if let Err(e) = utils::await_promise(writer.close()).await {
      return Err(WasmError::fail_to_close_writer(LOCATION, &format!("{:?}", e)));
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

  fn report_compress_progress(&self, number_of_compressed_files: usize, number_of_all_files: usize) -> Result<(), WasmError> {
    if let Some(compress_progress) = self.compress_progress.as_ref() {
      if let Err(e) = compress_progress.call2(
        &JsValue::NULL,
        &JsValue::from_f64(number_of_compressed_files as f64),
        &JsValue::from_f64(number_of_all_files as f64)
      ) {
        return Err(WasmError::fail_to_invoke_callback(utils::type_name(&Self::report_compress_progress), "compress_progress", &format!("{:?}", e)));
      }
    }
    Ok(())
  }

  fn report_average_speed(&self, total_bytes_written: u64, total_time_elapsed: f64) -> Result<(), WasmError> {
    if let Some(average_speed) = self.average_speed.as_ref() {
      if let Err(e) = average_speed.call2(
        &JsValue::NULL,
        &JsValue::from_f64(total_bytes_written as f64),
        &JsValue::from_f64(total_time_elapsed)
      ) {
        return Err(WasmError::fail_to_invoke_callback(utils::type_name(&Self::report_average_speed), "average_speed", &format!("{:?}", e)));
      }
    }
    Ok(())
  }

  fn report_current_speed(&self, delta_bytes_written: u64, delta_time_elapsed: f64) -> Result<(), WasmError> {
    if let Some(current_speed) = self.current_speed.as_ref() {
      if let Err(e) = current_speed.call2(
        &JsValue::NULL,
        &JsValue::from_f64(delta_bytes_written as f64),
        &JsValue::from_f64(delta_time_elapsed)
      ) {
        return Err(WasmError::fail_to_invoke_callback(utils::type_name(&Self::report_current_speed), "current_speed", &format!("{:?}", e)));
      }
    }
    Ok(())
  }

  fn report_current_file(&self, path: &str) -> Result<(), WasmError> {
    if let Some(current_file) = self.current_file.as_ref() {
      if let Err(e) = current_file.call1(&JsValue::NULL, &JsValue::from_str(path)) {
        return Err(WasmError::fail_to_invoke_callback(utils::type_name(&Self::report_current_file), "current_file", &format!("{:?}", e)));
      }
    }
    Ok(())
  }
}

#[wasm_bindgen]
impl Handles {
  /// Do a deep scan on input entries.
  ///
  /// # Parameters
  ///
  /// * `entries` - MUST be an array of `FileSystemEntry`
  ///
  /// # Returns
  ///
  /// - resolve: number of scanned entries
  /// - reject: a `WasmError` object
  pub async fn scan(&self, entries: js_sys::Array) -> Result<JsValue, WasmError> {
    self.context.scan_stage.file_path_list.borrow_mut().clear();
    self.context.scan_stage.file_system.borrow_mut().take();

    let first_entry = entries.get(0);

    if first_entry.eq(&JsValue::UNDEFINED) {
      return Err(WasmError::empty_file_list(utils::type_name(&Handles::scan)));
    }

    self.context.scan_stage.file_system.replace(Some(first_entry.unchecked_into::<web_sys::FileSystemEntry>().filesystem()));

    self.scan_internal(entries).await?;

    Ok(JsValue::from_f64(self.context.scan_stage.file_path_list.borrow().len() as f64))
  }

  /// Compress scanned entries.
  ///
  /// # Parameters
  ///
  /// * `output_file_name` - name of output zip file
  /// * `compression_level` - an integer, minimal 0, maximum 9
  ///
  /// # Returns
  ///
  /// - resolve: undefined
  /// - reject: a `WasmError` object
  pub async fn compress(&self, output_file_name: String, compression_level: u32) -> Result<JsValue, WasmError> {
    self.compress_internal(output_file_name, compression_level, None).await
  }

  /// Compress scanned entries and transform output bytes.
  ///
  /// # Parameters
  ///
  /// * `output_file_name` - name of output zip file
  /// * `compression_level` - an integer, minimal 0, maximum 9
  /// * `transform_script` - NOT IMPLEMENTED
  ///
  /// # Returns
  ///
  /// - resolve: undefined
  /// - reject: a `WasmError` object
  pub async fn compress_transform(&self, output_file_name: String, compression_level: u32, transform_script: String) -> Result<JsValue, WasmError> {
    self.compress_internal(output_file_name, compression_level, Some(transform_script)).await
  }

  /// Just transform the input file, only one file could be accepted.
  ///
  /// # Parameters
  ///
  /// * `transform_script` - NOT IMPLEMENTED
  ///
  /// # Returns
  ///
  /// - resolve: undefined
  /// - reject: a `WasmError` object
  pub async fn transform(&self, _transform_script: String) -> Result<JsValue, WasmError> {
    #[allow(non_snake_case)]
    let LOCATION = utils::type_name(&Self::recover);

    let file_path_list = self.context.scan_stage.file_path_list.borrow();
    if file_path_list.len() == 0 {
      return Err(WasmError::empty_file_list(LOCATION));
    }
    if file_path_list.len() > 1 {
      return Err(WasmError::too_many_files(LOCATION));
    }
    let FilePath { path, is_dir } = file_path_list.first().unwrap();

    if *is_dir {
      return Err(WasmError::can_not_transform_directory(LOCATION, path));
    }

    let file_system = self.context.scan_stage.file_system.borrow();
    let Some(file_system) = file_system.as_ref() else {
      return Err(WasmError::missing_file_system(LOCATION));
    };

    let file_entry = match js_futures::FileSystemDirectoryEntry_getFile_future::from(file_system.root(), path).await {
      Ok(entry) => entry,
      Err(e) => return Err(WasmError::fail_to_get_file_entry(LOCATION, &format!("{:?}", e))),
    };

    let file = match js_futures::FileSystemFileEntry_file_future::from(file_entry).await {
      Ok(file) => file,
      Err(e) => return Err(WasmError::fail_to_get_file(LOCATION, &format!("{:?}", e))),
    };

    let file_name = file.name();

    // cast File to Blob
    let Ok(blob) = file.dyn_into::<web_sys::Blob>() else {
      return Err(WasmError::dynamic_cast_error(LOCATION, "File", "Blob"));
    };

    // get BYOB reader
    let stream = blob.stream();
    let get_reader_option = web_sys::ReadableStreamGetReaderOptions::new();
    get_reader_option.set_mode(web_sys::ReadableStreamReaderMode::Byob);
    let Ok(reader) = stream.get_reader_with_options(&get_reader_option).dyn_into::<web_sys::ReadableStreamByobReader>() else {
      return Err(WasmError::dynamic_cast_error(LOCATION, "Object", "ReadableStreamByobReader"));
    };

    let writer = match self.create_writer.call1(&JsValue::NULL, &JsValue::from_str(&file_name)) {
      Ok(w) => w,
      Err(e) => return Err(WasmError::fail_to_create_writer(LOCATION, &format!("{:?}", e))),
    };
    let Ok(writer) = writer.dyn_into::<web_sys::WritableStreamDefaultWriter>() else {
      return Err(WasmError::dynamic_cast_error(LOCATION, "JsValue", "WritableStreamDefaultWriter"));
    };

    let mut buffer_read = vec![0_u8; 16 * 1024 * 1024];
    let mut array_buffer_read = js_sys::ArrayBuffer::new(16 * 1024 * 1024);

    const TRANSFORM_BUFFER_WRITE_THRESHOLD: usize = 16 * 1024 * 1024;
    let mut transform_writer = TransformWriter::new(
      &writer,
      TRANSFORM_BUFFER_WRITE_THRESHOLD,
      8,
      16 * 1024 * 1024,
      false,
    );

    loop {
      // js reader -> js buffer
      let ReadResult { new_buffer: array_buffer_new_read, view: view_read, done } = utils::byob_read(&array_buffer_read, &reader).await?;

      let bytes_read = view_read.byte_length();

      array_buffer_read = array_buffer_new_read; // replace old detached ArrayBuffer immediately

      // if indeed read some bytes
      if bytes_read > 0 {
        // js buffer -> wasm buffer slice
        let wasm_slice = &mut buffer_read[0..bytes_read as usize];
        view_read.copy_to(wasm_slice);

        // wasm buffer slice -> transform writer
        transform_writer.write(wasm_slice).unwrap();

        transform_writer.dump(true).await?;
      }

      if done {
        break;
      }
    }

    transform_writer.flush().unwrap();

    transform_writer.dump(false).await?;

    if let Err(e) = utils::await_promise(writer.close()).await {
      return Err(WasmError::fail_to_close_writer(LOCATION, &format!("{:?}", e)));
    }

    Ok(JsValue::UNDEFINED)
  }

  /// Recover the input file, only one file could be accepted.
  ///
  /// # Parameters
  ///
  /// * `transform_script` - NOT IMPLEMENTED
  ///
  /// # Returns
  ///
  /// - resolve: undefined
  /// - reject: a `WasmError` object
  pub async fn recover(&self, _transform_script: String) -> Result<JsValue, WasmError> {
    #[allow(non_snake_case)]
    let LOCATION = utils::type_name(&Self::recover);

    let file_path_list = self.context.scan_stage.file_path_list.borrow();
    if file_path_list.len() == 0 {
      return Err(WasmError::empty_file_list(LOCATION));
    }
    if file_path_list.len() > 1 {
      return Err(WasmError::too_many_files(LOCATION));
    }
    let FilePath { path, is_dir } = file_path_list.first().unwrap();

    if *is_dir {
      return Err(WasmError::can_not_recover_directory(LOCATION, path));
    }

    let file_system = self.context.scan_stage.file_system.borrow();
    let Some(file_system) = file_system.as_ref() else {
      return Err(WasmError::missing_file_system(LOCATION));
    };

    let file_entry = match js_futures::FileSystemDirectoryEntry_getFile_future::from(file_system.root(), path).await {
      Ok(entry) => entry,
      Err(e) => return Err(WasmError::fail_to_get_file_entry(LOCATION, &format!("{:?}", e))),
    };

    let file = match js_futures::FileSystemFileEntry_file_future::from(file_entry).await {
      Ok(file) => file,
      Err(e) => return Err(WasmError::fail_to_get_file(LOCATION, &format!("{:?}", e))),
    };

    let file_name = file.name();

    // cast File to Blob
    let Ok(blob) = file.dyn_into::<web_sys::Blob>() else {
      return Err(WasmError::dynamic_cast_error(LOCATION, "File", "Blob"));
    };

    // get BYOB reader
    let stream = blob.stream();
    let get_reader_option = web_sys::ReadableStreamGetReaderOptions::new();
    get_reader_option.set_mode(web_sys::ReadableStreamReaderMode::Byob);
    let Ok(reader) = stream.get_reader_with_options(&get_reader_option).dyn_into::<web_sys::ReadableStreamByobReader>() else {
      return Err(WasmError::dynamic_cast_error(LOCATION, "Object", "ReadableStreamByobReader"));
    };

    let writer = match self.create_writer.call1(&JsValue::NULL, &JsValue::from_str(&file_name)) {
      Ok(w) => w,
      Err(e) => return Err(WasmError::fail_to_create_writer(LOCATION, &format!("{:?}", e))),
    };
    let Ok(writer) = writer.dyn_into::<web_sys::WritableStreamDefaultWriter>() else {
      return Err(WasmError::dynamic_cast_error(LOCATION, "JsValue", "WritableStreamDefaultWriter"));
    };

    let mut buffer_read = vec![0_u8; 16 * 1024 * 1024];
    let mut array_buffer_read = js_sys::ArrayBuffer::new(16 * 1024 * 1024);

    const RECOVER_BUFFER_WRITE_THRESHOLD: usize = 16 * 1024 * 1024;
    let mut recover_writer = RecoverWriter::new(
      &writer,
      RECOVER_BUFFER_WRITE_THRESHOLD,
      8,
      16 * 1024 * 1024,
      false,
    );

    loop {
      // js reader -> js buffer
      let ReadResult { new_buffer: array_buffer_new_read, view: view_read, done } = utils::byob_read(&array_buffer_read, &reader).await?;

      let bytes_read = view_read.byte_length();

      array_buffer_read = array_buffer_new_read; // replace old detached ArrayBuffer immediately

      // if indeed read some bytes
      if bytes_read > 0 {
        // js buffer -> wasm buffer slice
        let wasm_slice = &mut buffer_read[0..bytes_read as usize];
        view_read.copy_to(wasm_slice);

        // wasm buffer slice -> transform writer
        recover_writer.write(wasm_slice).unwrap();

        recover_writer.dump(true).await?;
      }

      if done {
        break;
      }
    }

    recover_writer.flush().unwrap();

    recover_writer.dump(false).await?;

    if let Err(e) = utils::await_promise(writer.close()).await {
      return Err(WasmError::fail_to_close_writer(LOCATION, &format!("{:?}", e)));
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

  /// Register "compress_progress" callback.
  ///
  /// # Parameters
  ///
  /// * `callback` - a function like `(number_of_compressed_files: number, number_of_all_files: number) => {}`
  pub fn register_compress_progress(&mut self, callback: js_sys::Function) {
    self.compress_progress = Some(callback);
  }

  /// Register "average_speed" callback.
  ///
  /// # Parameters
  ///
  /// * `callback` - a function like `(total_bytes_written: number, total_time_elapsed: number) => {}`
  pub fn register_average_speed(&mut self, callback: js_sys::Function) {
    self.average_speed = Some(callback);
  }

  /// Register "current_speed" callback.
  ///
  /// # Parameters
  ///
  /// * `callback` - a function like `(delta_bytes_written: number, delta_time_elapsed: number) => {}`
  pub fn register_current_speed(&mut self, callback: js_sys::Function) {
    self.current_speed = Some(callback);
  }

  /// Register "current_current_filespeed" callback.
  ///
  /// # Parameters
  ///
  /// * `callback` - a function like `(path: string) => {}`
  pub fn register_current_file(&mut self, callback: js_sys::Function) {
    self.current_file = Some(callback);
  }
}
