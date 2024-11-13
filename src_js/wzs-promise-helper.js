if (!window.wzs) {
  window.wzs = {};
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
