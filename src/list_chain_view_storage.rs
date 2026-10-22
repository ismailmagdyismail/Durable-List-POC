use std::path::PathBuf;

use crate::{
    append_storage_writer::AppendStorageWriter, in_place_storage_writer::InPlaceStorageWriter,
    list_chain_node_codec::ListChainNodeCodec, sequential_cursor::SequentialCursor,
    storage_errors::StorageError,
};

pub const LIST_CHAIN_FILE_NAME: &str = "list_chain.dat";

#[derive(Debug, PartialEq, Eq)]
pub struct ListChainNode {
    pub prev_offset: u64,
    pub node_offset: u64,
    pub next_offset: u64,
}

#[derive(Debug)]
pub struct ListChainViewStorage {
    file_path: PathBuf,
    sequential_writer: AppendStorageWriter,
    in_place_writer: InPlaceStorageWriter,
}

impl ListChainViewStorage {
    pub fn create_list_chain_file_path(storage_dir: &str) -> PathBuf {
        let mut file_path = PathBuf::from(storage_dir);
        file_path.push(LIST_CHAIN_FILE_NAME);
        file_path
    }

    pub fn open(file_path: PathBuf) -> Result<Self, StorageError> {
        let sequential_writer = AppendStorageWriter::open(file_path.clone())?;
        let in_place_writer = InPlaceStorageWriter::open(file_path.clone())?;

        Ok(Self {
            file_path,
            sequential_writer,
            in_place_writer,
        })
    }

    pub fn append(&mut self, nodes: &[ListChainNode]) -> Result<u64, StorageError> {
        let mut bytes = Vec::with_capacity(nodes.len() * ListChainNodeCodec::RECORD_SIZE);
        for node in nodes {
            bytes.extend_from_slice(&ListChainNodeCodec::encode_record(node));
        }

        self.sequential_writer.append(&bytes)
    }

    pub fn overwrite(&mut self, offset: u64, node: &ListChainNode) -> Result<(), StorageError> {
        self.sequential_writer.flush()?;
        self.in_place_writer
            .overwrite(offset, &ListChainNodeCodec::encode_record(node))
    }

    pub fn sync(&mut self) -> Result<(), StorageError> {
        self.sequential_writer.sync()?;
        self.in_place_writer.sync()
    }

    pub fn cursor(&mut self) -> Result<ListChainViewCursor, StorageError> {
        self.sequential_writer.flush()?;
        ListChainViewCursor::open(self.file_path.clone())
    }
}

pub struct ListChainViewCursor {
    cursor: SequentialCursor,
}

impl ListChainViewCursor {
    pub fn open(file_path: PathBuf) -> Result<Self, StorageError> {
        Ok(Self {
            cursor: SequentialCursor::open(file_path)?,
        })
    }

    pub fn next_record(&mut self) -> Result<Option<ListChainNode>, StorageError> {
        let Some(bytes) = self.cursor.next_record(ListChainNodeCodec::RECORD_SIZE)? else {
            return Ok(None);
        };

        ListChainNodeCodec::decode_record(&bytes).map(Some)
    }
}

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
    fn creates_list_chain_path_in_storage_directory() {
        assert_eq!(
            ListChainViewStorage::create_list_chain_file_path("storage"),
            PathBuf::from("storage").join(LIST_CHAIN_FILE_NAME)
        );
    }

    #[test]
    fn appends_overwrites_and_reads_list_chain_records_through_the_facade() {
        let file_path = test_file_path("list-chain");
        let first = ListChainNode {
            prev_offset: 0,
            node_offset: 4,
            next_offset: ListChainNodeCodec::RECORD_SIZE as u64,
        };
        let second = ListChainNode {
            prev_offset: 0,
            node_offset: 8,
            next_offset: 0,
        };
        let replacement = ListChainNode {
            prev_offset: 0,
            node_offset: 12,
            next_offset: ListChainNodeCodec::RECORD_SIZE as u64,
        };
        let mut storage = ListChainViewStorage::open(file_path.clone()).unwrap();

        assert_eq!(storage.append(&[first, second]).unwrap(), 0);
        storage.overwrite(0, &replacement).unwrap();
        storage.sync().unwrap();

        let mut cursor = storage.cursor().unwrap();
        assert_eq!(cursor.next_record().unwrap(), Some(replacement));
        assert_eq!(
            cursor.next_record().unwrap(),
            Some(ListChainNode {
                prev_offset: 0,
                node_offset: 8,
                next_offset: 0,
            })
        );
        assert_eq!(cursor.next_record().unwrap(), None);

        drop(cursor);
        drop(storage);
        fs::remove_file(file_path).unwrap();
    }
}
