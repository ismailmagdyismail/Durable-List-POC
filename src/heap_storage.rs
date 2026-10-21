use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Seek, SeekFrom, Write},
};

use crate::{heap::create_heap_file_path, storage_errors::StorageError};

#[derive(Debug)]
pub struct HeapStorage {
    buff_writer: BufWriter<File>,
}

pub const RECORD_SIZE: u8 = 4;

impl HeapStorage {
    pub fn new(storage_dir: &str) -> Result<HeapStorage, StorageError> {
        let file_path = create_heap_file_path(storage_dir);
        let mut options = OpenOptions::new();
        options.create(true).append(true).write(true);
        let file = options
            .open(file_path)
            .map_err(|err| StorageError::HeadDataFile(err.to_string()))?;

        let mut buff_writer = BufWriter::new(file);
        buff_writer
            .seek(SeekFrom::End(0))
            .map_err(|err| StorageError::HeapSeek(err.to_string()))?;
        let heap_storage = HeapStorage { buff_writer };

        Ok(heap_storage)
    }

    pub fn append_records_batch(&mut self, records: &[i32]) -> Result<u64, StorageError> {
        let mut encoded_buffer = Vec::new();
        let required_size = records.len() * RECORD_SIZE as usize;
        encoded_buffer.resize(required_size, b'0');

        // encoding
        for (i, record) in records.iter().enumerate() {
            let start = i * RECORD_SIZE as usize;
            let end = start + RECORD_SIZE as usize;
            let slice = &mut encoded_buffer[start..end];
            slice.copy_from_slice(&record.to_le_bytes());
        }

        // get start offset of first record of the whole batch
        let starting_offset = self
            .buff_writer
            .stream_position()
            .map_err(|err| StorageError::HeapSeek(err.to_string()))?;

        // write to os-page cache (since not O_DIRECT / direct-IO yet)
        self.buff_writer
            .write_all(&encoded_buffer)
            .map_err(|err| StorageError::HeapAppend(err.to_string()))?;

        Ok(starting_offset)
    }

    pub fn sync(&mut self) -> Result<(), StorageError> {
        self.buff_writer
            .flush()
            .map_err(|err| StorageError::HeapFlush(err.to_string()))?;
        self.buff_writer
            .get_mut()
            .sync_all()
            .map_err(|err| StorageError::HeapFSync(err.to_string()))?;
        Ok(())
    }
}
