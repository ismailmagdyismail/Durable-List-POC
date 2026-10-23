use std::{
    fs::File,
    io::{BufReader, Read, Seek},
    path::PathBuf,
};

use crate::{
    append_storage_writer::AppendStorageWriter, heap_records_codec::HeapRecordsCodec,
    sequential_cursor::SequentialCursor, storage_errors::StorageError,
};

#[derive(Debug)]
pub struct HeapStorage {
    file_path: PathBuf,
    access: AppendStorageWriter,
    random_read_buffer: BufReader<File>,
}

impl HeapStorage {
    pub fn create_heap_file_path(storage_dir: &str) -> PathBuf {
        let mut file_path = PathBuf::from(storage_dir);
        file_path.push(HEAP_FILE_NAME);
        file_path
    }

    pub fn open(file_path: PathBuf) -> Result<Self, StorageError> {
        let access = AppendStorageWriter::open(file_path.clone())?;

        let heap_file: File = File::open(&file_path).map_err(|_| {
            StorageError::Open("Cannot open Heap file for Random reads".to_string())
        })?;
        let random_reader = BufReader::new(heap_file);

        Ok(Self {
            file_path,
            access,
            random_read_buffer: random_reader,
        })
    }

    pub fn append(&mut self, records: &[i32]) -> Result<u64, StorageError> {
        let mut bytes = Vec::with_capacity(records.len() * HeapRecordsCodec::RECORD_SIZE);
        for record in records {
            bytes.extend_from_slice(&HeapRecordsCodec::encode_record(*record));
        }

        self.access.append(&bytes)
    }

    pub fn read_record(&mut self, offset: u64) -> Result<i32, StorageError> {
        let mut buffer = Vec::new();
        buffer.resize(HeapRecordsCodec::RECORD_SIZE, b'0');

        self.random_read_buffer
            .seek(std::io::SeekFrom::Start(offset))
            .map_err(|err| {
                let fmt_error = format!("Error in MetaData Random Read Seek {}", err);
                StorageError::Seek(fmt_error)
            })?;

        self.random_read_buffer
            .read_exact(&mut buffer)
            .map_err(|err| StorageError::HeapCorruptedRecord(err.to_string()))?;

        let record = HeapRecordsCodec::decode_record(&buffer)?;
        Ok(record)
    }

    pub fn sync(&mut self) -> Result<(), StorageError> {
        self.access.sync()
    }

    pub fn cursor(&mut self) -> Result<HeapCursor, StorageError> {
        self.access.flush()?;
        HeapCursor::open(self.file_path.clone())
    }
}

pub struct HeapCursor {
    cursor: SequentialCursor,
}

impl HeapCursor {
    pub fn open(file_path: PathBuf) -> Result<Self, StorageError> {
        Ok(Self {
            cursor: SequentialCursor::open(file_path)?,
        })
    }

    pub fn next_record(&mut self) -> Result<Option<i32>, StorageError> {
        let Some(bytes) = self.cursor.next_record(HeapRecordsCodec::RECORD_SIZE)? else {
            return Ok(None);
        };

        HeapRecordsCodec::decode_record(&bytes).map(Some)
    }
}

pub const HEAP_FILE_NAME: &str = "heap.dat";

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    fn test_file_path(name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!(
            "durable-linked-list-{name}-{}-{timestamp}.dat",
            std::process::id()
        ))
    }

    #[test]
    fn creates_heap_path_in_storage_directory() {
        assert_eq!(
            HeapStorage::create_heap_file_path("storage"),
            PathBuf::from("storage").join(HEAP_FILE_NAME)
        );
    }

    #[test]
    fn appends_and_reads_heap_records_through_the_facade() {
        let file_path = test_file_path("heap");
        let mut storage = HeapStorage::open(file_path.clone()).unwrap();

        assert_eq!(storage.append(&[1, -2]).unwrap(), 0);
        assert_eq!(storage.append(&[3]).unwrap(), 8);
        storage.sync().unwrap();

        let mut cursor = storage.cursor().unwrap();
        assert_eq!(cursor.next_record().unwrap(), Some(1));
        assert_eq!(cursor.next_record().unwrap(), Some(-2));
        assert_eq!(cursor.next_record().unwrap(), Some(3));
        assert_eq!(cursor.next_record().unwrap(), None);

        drop(cursor);
        drop(storage);
        fs::remove_file(file_path).unwrap();
    }
}
