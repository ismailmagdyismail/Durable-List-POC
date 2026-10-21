use std::{
    fs::{File, OpenOptions},
    io::{BufReader, Read},
    path::PathBuf,
};

use crate::{sequential_access::SequentialReader, storage_errors::StorageError};

pub struct SequentialCursor<T>
where
    T: SequentialReader,
{
    buff_reader: BufReader<File>,
    reader: T,
}

impl<T> SequentialCursor<T>
where
    T: SequentialReader,
    T::ReaderType: SequentialReader,
{
    pub fn new(
        file_path: PathBuf,
    ) -> Result<SequentialCursor<<T as SequentialReader>::ReaderType>, StorageError> {
        let mut options = OpenOptions::new();
        let file = options.read(true).open(file_path).map_err(|err| {
            return StorageError::HeadDataFile(err.to_string());
        })?;
        let buff_reader = BufReader::new(file);
        let cursor = SequentialCursor {
            buff_reader,
            reader: T::new(),
        };
        Ok(cursor)
    }

    pub fn next_record(&mut self) -> Result<Option<T::RecordType>, StorageError> {
        let mut buffer = Vec::new();
        let record_size = self.reader.get_record_size() as usize;
        buffer.resize(record_size as usize, b'0');

        let size = self
            .buff_reader
            .read(&mut buffer)
            .map_err(|err| StorageError::HeapRead(err.to_string()))?;

        if size == 0 {
            return Ok(None);
        }

        assert!(size == record_size);

        let decoded_record = self.reader.decode_record(&buffer)?;

        Ok(Some(decoded_record))
    }
}
