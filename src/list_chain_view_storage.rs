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
        let mut sequential_writer = AppendStorageWriter::open(file_path.clone())?;
        let mut in_place_writer = InPlaceStorageWriter::open(file_path.clone())?;
        Self::zero_out_0th_index(&mut in_place_writer, &mut sequential_writer)?;

        Ok(Self {
            file_path,
            sequential_writer,
            in_place_writer,
        })
    }

    pub fn nth_node_offset(node: u32) -> u64 {
        return ListChainNodeCodec::RECORD_SIZE as u64 * (node + 1) as u64;
    }

    pub fn get_nodes_count(&mut self) -> Result<u32, StorageError> {
        let file_end_offset = self.sequential_writer.get_offset()?;
        // skip first zeroed out null node at the start
        let count =
            (file_end_offset - Self::nth_node_offset(0)) / ListChainNodeCodec::RECORD_SIZE as u64;
        Ok(count as u32)
    }

    pub fn insert_node(&mut self, index: u32, node_offset: u64) -> Result<(), StorageError> {
        let nodes_count = self.get_nodes_count()?;
        if index < nodes_count {
            self.overwrite_node_offset(index, node_offset)?;
            Ok(())
        } else {
            self.append_node(node_offset)?;
            Ok(())
        }
    }

    fn append_node(&mut self, node_offset: u64) -> Result<u64, StorageError> {
        let final_node_offset = self.get_final_metadata_node_offset()?;

        // NOTE could be simplified since we gurantee 0th entry already exist
        // we could avoid branching

        // first node to be added
        let new_node_offset = if final_node_offset < Self::nth_node_offset(0) {
            // create chain between prev node, current new node, and null next entry
            let new_chain_node = ListChainNode {
                prev_offset: 0,
                node_offset: node_offset,
                next_offset: 0,
            };

            let mut bytes = Vec::with_capacity(ListChainNodeCodec::RECORD_SIZE);
            bytes.extend_from_slice(&ListChainNodeCodec::encode_record(&new_chain_node));

            // add new node
            let new_node_offset = self.sequential_writer.append(&bytes)?;
            new_node_offset
        } else {
            // create chain between prev node, current new node, and null next entry
            let new_chain_node = ListChainNode {
                prev_offset: final_node_offset,
                node_offset,
                next_offset: 0,
            };

            let mut bytes = Vec::with_capacity(ListChainNodeCodec::RECORD_SIZE);
            bytes.extend_from_slice(&ListChainNodeCodec::encode_record(&new_chain_node));

            // add new node
            let new_node_offset = self.sequential_writer.append(&bytes)?;

            // update previous node's Next_Offset
            let encoded_next_offset =
                ListChainNodeCodec::encode_record_prev_offset(new_node_offset);
            let offset_of_next_offset_entrt_of_prev_node =
                ListChainNodeCodec::calculate_next_offset(final_node_offset);
            self.in_place_writer.overwrite(
                offset_of_next_offset_entrt_of_prev_node,
                &encoded_next_offset,
            )?;

            new_node_offset
        };

        Ok(new_node_offset)
    }

    fn overwrite_node_offset(
        &mut self,
        index: u32,
        new_val_node_offset: u64,
    ) -> Result<(), StorageError> {
        // self.sequential_writer.flush()?;
        let metadata_node_offset = Self::nth_node_offset(index);
        let val_node_entry_offset = ListChainNodeCodec::calculate_node_offset(metadata_node_offset);

        let encoded_node_offset =
            ListChainNodeCodec::encode_record_prev_offset(new_val_node_offset);
        self.in_place_writer
            .overwrite(val_node_entry_offset, &encoded_node_offset)
    }

    pub fn sync(&mut self) -> Result<(), StorageError> {
        self.sequential_writer.sync()?;
        self.in_place_writer.sync()
    }

    pub fn cursor(&mut self) -> Result<ListChainViewCursor, StorageError> {
        self.sequential_writer.flush()?;
        ListChainViewCursor::open(self.file_path.clone())
    }

    fn get_final_metadata_node_offset(&mut self) -> Result<u64, StorageError> {
        let file_end_offset = self.sequential_writer.get_offset()?;
        if file_end_offset == 0 {
            return Ok(0);
        }
        let final_node_offset = file_end_offset - ListChainNodeCodec::RECORD_SIZE as u64;
        Ok(final_node_offset)
    }

    // 1-Based Indexing
    fn zero_out_0th_index(
        in_place_writer: &mut InPlaceStorageWriter,
        seq_writer: &mut AppendStorageWriter,
    ) -> Result<(), StorageError> {
        let mut bytes = Vec::with_capacity(ListChainNodeCodec::RECORD_SIZE);
        let node = ListChainNode {
            prev_offset: 0,
            next_offset: 0,
            node_offset: 0,
        };
        bytes.extend_from_slice(&ListChainNodeCodec::encode_record(&node));

        if seq_writer.get_offset()? == 0 {
            seq_writer.append(&bytes)?;
            seq_writer.flush()?;
            seq_writer.sync()?;
        } else {
            in_place_writer.overwrite(0, &bytes)?;
            in_place_writer.sync()?;
        }

        Ok(())
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
    fn test_node_count_calculation() {
        let file_path = test_file_path("list-chain");
        let mut storage = ListChainViewStorage::open(file_path.clone()).unwrap();

        assert_eq!(storage.get_nodes_count().unwrap(), 0);

        storage.insert_node(0, 0).unwrap();
        assert_eq!(storage.get_nodes_count().unwrap(), 1);

        storage.insert_node(1, 4).unwrap();
        assert_eq!(storage.get_nodes_count().unwrap(), 2);

        storage.insert_node(0, 8).unwrap();
        assert_eq!(storage.get_nodes_count().unwrap(), 2);
    }

    #[test]
    fn creates_list_chain_path_in_storage_directory() {
        assert_eq!(
            ListChainViewStorage::create_list_chain_file_path("storage"),
            PathBuf::from("storage").join(LIST_CHAIN_FILE_NAME)
        );
    }

    #[test]
    fn appends_and_reads_list_chain_records_through_the_facade() {
        let file_path = test_file_path("list-chain");
        let first = ListChainNode {
            prev_offset: 0,
            node_offset: 4,
            next_offset: ListChainViewStorage::nth_node_offset(1),
        };
        let second = ListChainNode {
            prev_offset: ListChainViewStorage::nth_node_offset(0),
            node_offset: 8,
            next_offset: 0,
        };
        let zero_node = ListChainNode {
            prev_offset: 0,
            node_offset: 0,
            next_offset: 0,
        };
        let mut storage = ListChainViewStorage::open(file_path.clone()).unwrap();

        assert_eq!(
            storage.append_node(first.node_offset).unwrap(),
            ListChainViewStorage::nth_node_offset(0),
        );
        assert_eq!(
            storage.append_node(second.node_offset).unwrap(),
            ListChainViewStorage::nth_node_offset(1),
        );
        storage.sync().unwrap();

        let mut cursor = storage.cursor().unwrap();
        assert_eq!(cursor.next_record().unwrap(), Some(zero_node));
        assert_eq!(cursor.next_record().unwrap(), Some(first));
        assert_eq!(
            cursor.next_record().unwrap(),
            Some(ListChainNode {
                prev_offset: ListChainViewStorage::nth_node_offset(0),
                node_offset: second.node_offset,
                next_offset: 0,
            })
        );
        assert_eq!(cursor.next_record().unwrap(), None);

        drop(cursor);
        drop(storage);
        fs::remove_file(file_path).unwrap();
    }
}
