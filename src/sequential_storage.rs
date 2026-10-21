use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Seek, SeekFrom, Write},
    path::PathBuf,
};

use crate::{sequential_access::SequentialWriter, storage_errors::StorageError};

#[derive(Debug)]
pub struct SequentialStorage<T>
where
    T: SequentialWriter,
{
    buff_writer: BufWriter<File>,
    writer: T,
}

impl<T> SequentialStorage<T>
where
    T: SequentialWriter,
    T::ReaderType: SequentialWriter,
{
    pub fn new(
        file_path: PathBuf,
    ) -> Result<SequentialStorage<<T as SequentialWriter>::ReaderType>, StorageError> {
        let mut options = OpenOptions::new();
        options.create(true).append(true).write(true);
        let file = options
            .open(file_path)
            .map_err(|err| StorageError::HeadDataFile(err.to_string()))?;

        let mut buff_writer = BufWriter::new(file);
        buff_writer
            .seek(SeekFrom::End(0))
            .map_err(|err| StorageError::HeapSeek(err.to_string()))?;
        let heap_storage = SequentialStorage {
            buff_writer,
            writer: T::new(),
        };

        Ok(heap_storage)
    }

    pub fn append_records_batch(&mut self, records: &[T::RecordType]) -> Result<u64, StorageError> {
        let mut encoded_buffer = Vec::new();
        let record_size = self.writer.get_record_size() as usize;
        let required_size = records.len() * record_size;
        encoded_buffer.resize(required_size, b'0');

        // encoding
        for (i, record) in records.iter().enumerate() {
            let start = i * record_size;
            let end = start + record_size;
            let slice = &mut encoded_buffer[start..end];
            self.writer.encode_record(record, slice);
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
