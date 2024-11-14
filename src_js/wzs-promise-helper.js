if (!window.wzs) {
  window.wzs = {};
}

if (!window.wzs.promises) {
  window.wzs.promises = {};
}

/**
 * Wrap FileSystemFileEntry.file() into Promise
 * @param {FileSystemFileEntry} file_entry
 */
window.wzs.file_system_file_entry__file = file_entry => new Promise((resolve, reject) => file_entry.file(resolve, reject));

/**
 * Wrap FileSystemDirectoryReader.readEntries() in Promise
 * @param {FileSystemDirectoryReader} directory_reader
 */
window.wzs.file_system_directory_reader__read_entries = directory_reader => new Promise((resolve, reject) => directory_reader.readEntries(resolve, reject));

window.wzs.promises.create = (id) => new Promise((resolve, reject) => window.wzs.promises[id] = { resolve, reject });

window.wzs.promises.resolve = (id) => {
  window.wzs.promises[id].resolve();
  delete window.wzs.promises[id];
};

window.wzs.promises.reject = (id, err) => {
  window.wzs.promises[id].reject(err);
  delete window.wzs.promises[id];
};
