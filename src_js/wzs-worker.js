postMessage({
  generic_message_type: 'worker',
  worker_message_type: 'loaded',
});

addEventListener('message', async ({ data }) => {
  if (data.generic_message_type !== 'worker') {
    return;
  }

  if (data.worker_message_type === 'initialize_wasm') {
    try {
      let module = await import(data.worker_wasm_path);

      await module.default();

      postMessage({
        generic_message_type: 'worker',
        worker_message_type: 'initialize_wasm_success',
      });
    } catch (err) {
      postMessage({
        generic_message_type: 'worker',
        worker_message_type: 'initialize_wasm_fail',
        detail: err,
      });
    }
  }
});
