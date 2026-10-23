use std::path::PathBuf;

use crate::{
    heap_storage::HeapStorage, list_chain_view_storage::ListChainViewStorage,
    storage_errors::StorageError,
};

pub struct ImmutableStorageEngine {
    heap_storage: HeapStorage,
    metadata_storage: ListChainViewStorage,
}

impl ImmutableStorageEngine {
    pub fn new(db_path: PathBuf) -> Result<ImmutableStorageEngine, StorageError> {
        let path_str = db_path.to_str();
        if path_str.is_none() {
            return Err(StorageError::Open("Empty DB Directory path".to_string()));
        }
        let path_str = path_str.unwrap();
        std::fs::create_dir_all(path_str).map_err(|err| {
            let fmt_error = format!("DB Directory couldn't be created {}", err);
            StorageError::Open(fmt_error)
        })?;
        let engine = ImmutableStorageEngine {
            heap_storage: HeapStorage::open(HeapStorage::create_heap_file_path(path_str))?,
            metadata_storage: ListChainViewStorage::open(
                ListChainViewStorage::create_list_chain_file_path(path_str),
            )?,
        };
        Ok(engine)
    }

    pub fn get_nodes_count(&mut self) -> u32 {
        self.metadata_storage.get_nodes_count().unwrap()
    }

    pub fn write_nodes_batch(&mut self, nodes: &[(u32, i32)]) -> Result<(), StorageError> {
        for (idx, val) in nodes {
            let offset = self.heap_storage.append(&[*val])?;
            self.metadata_storage.insert_node(*idx, offset)?;
        }
        self.sync()?;
        Ok(())
    }

    pub fn write_node(&mut self, idx: u32, val: i32) -> Result<(), StorageError> {
        let offset = self.heap_storage.append(&[val])?;
        self.metadata_storage.insert_node(idx, offset)?;
        self.sync()?;
        Ok(())
    }

    pub fn read_node(&mut self, idx: u32) -> Result<Option<i32>, StorageError> {
        let node = self.metadata_storage.get_metadata_node(idx)?;
        if node.is_none() {
            return Ok(None);
        }
        let node = node.unwrap();
        let node_val = self.heap_storage.read_record(node.node_offset)?;
        Ok(Some(node_val))
    }

    fn sync(&mut self) -> Result<(), StorageError> {
        self.heap_storage.sync()?;
        self.metadata_storage.sync()?;
        Ok(())
    }
}
