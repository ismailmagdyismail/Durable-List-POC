use std::path::PathBuf;

pub const HEAP_FILE_NAME: &'static str = "heap.dat";

pub fn create_heap_file_path(storage_dir: &str) -> PathBuf {
    let mut file_path = PathBuf::from(storage_dir);
    file_path.push(HEAP_FILE_NAME);
    file_path
}
