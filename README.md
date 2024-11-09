# WASM ZIP STREAM

A stream style zip compressor running in browser.

## build

wasm-pack build --release --target web

## Usage

This library allows users to compress a lot of files into a very big zip file and download it locally. An another library, `StreamSaver.js`,
is required as the runtime dependency of this library, it is because `StreamSaver.js` can produce the required `WritableStreamDefaultWriter`
that will be used when creating the file writer.

`StreamSaver.js` can be found on `https://github.com/jimmywarting/StreamSaver.js`.

The presence of `StreamSaver.js` also means that you will need a HTTPS environment.

### initialize_context() and Handles

The exported function `initialize_context()` will return a `Handles` object, which has some methods you can use:
- `scan` accepts an array of `FileSystemEntry`s, which you can get from the `dataTransfer` property in `DargEvent`.
- `compress` accepts a file name and a compression level, this method will create a writer, compress scanned files into a zip file stream,
  and use that writer to write the stream to user's file system.
- `register_scan_progress` accepts a callback like `(number_of_scanned_entries: number) => {}` for each encountered file.
- `register_compress_progress` accepts a callback like `(number_of_compressed_files: number, number_of_all_files: number) => {}`.
- `register_average_speed` accepts a callback like `(total_bytes_written: number, total_time_elapsed: number) => {}`.
- `register_current_speed` accepts a callback like `(delta_bytes_written: number, delta_time_elapsed: number) => {}`.
- `register_current_file` accepts a callback like `(path: string) => {}`.

For detailed description please refer to docs.rs.

## Example

A working example could be found [here](https://wasm-zip-stream-example.saya.pw), you can inspect files using devtool or find files under the `example` folder in github repo.
