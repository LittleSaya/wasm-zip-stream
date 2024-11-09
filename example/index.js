import init, * as wasm from './wasm_zip_stream.js';

const streamSaver = window.streamSaver;
streamSaver.mitm = 'https://wasm-zip-stream-example.saya.pw/mitm.html';

/**
 * @param {wasm.WasmError} wasm_error
 */
function alert_wasm_error(title, wasm_error) {
    alert(title + '\r\ncode = ' + wasm_error.code + '\r\narg0 = ' + wasm_error.arg0 + '\r\narg1 = ' + wasm_error.arg1 + '\r\narg2 = ' + wasm_error.arg2 + '\r\narg3 = ' + wasm_error.arg3);
}

document.addEventListener('DOMContentLoaded', async () => {
    try {
        // initialize wasm
        await init();

        // initialize context and save the returned Handles object
        let handles = wasm.initialize_context(
            // this library itself does not know how to create file writer, so we pass a callback function here,
            // in this function, we will invoke StreamSaver.js to create the writer.
            file_name => {
                const output_stream = streamSaver.createWriteStream(file_name, {
                    size: undefined,
                    writableStrategy: undefined,
                    readableStrategy: undefined
                });
                const writer = output_stream.getWriter();
                return writer;
            }
        );

        // setup the dropping area
        let dropping_area = document.querySelector('div#dropping_area');

        dropping_area.addEventListener('dragover', ev => {
            ev.preventDefault();
            ev.dataTransfer.dropEffect = 'move';
        });

        dropping_area.addEventListener('drop', async ev => {
            ev.preventDefault();

            // convert DataTransferItem to FileSystemEntry
            let entries = [];
            for (let item of ev.dataTransfer.items) {
                entries.push(item.webkitGetAsEntry());
            }

            // invoke the `scan` method
            try {
                let number_of_files = await handles.scan(entries);
                alert('scan complete, found ' + number_of_files + ' files');
            }
            catch (err) {
                // the err has type WasmError
                alert_wasm_error('fail to scan', err);
            }
        });

        // setup the compress button
        let compress = document.querySelector('button#compress');

        compress.addEventListener('click', async ev => {
            try {
                let compression_level = 5;
                let compressed_file_name = 'change_this.zip';
                await handles.compress(compressed_file_name, compression_level);
            }
            catch (err) {
                // the err has type WasmError
                alert_wasm_error('fail to compress', err);
            }
        });
    }
    catch (err) {
        alert('fail to initialize');
    }
});
