use std::{collections::BTreeMap, sync::Arc};

use crate::flusher::FlushWorker;

pub struct WriteBuffer {
    active_buffer: Box<BTreeMap<u32, i32>>,
    bg_buffer: Arc<Box<BTreeMap<u32, i32>>>,
    max_buffer_size: usize,
    bg_flusher: FlushWorker,
}

impl WriteBuffer {
    pub fn new(max_buffer_size: usize, bg_flusher: FlushWorker) -> Self {
        let active_buffer = Box::new(BTreeMap::new());
        let bg_buffer = Arc::new(Box::new(BTreeMap::new()));
        WriteBuffer {
            active_buffer,
            max_buffer_size,
            bg_flusher,
            bg_buffer,
        }
    }

    pub fn insert_node(&mut self, idx: u32, val: i32) {
        self.active_buffer.insert(idx, val);

        if self.active_buffer.len() >= self.max_buffer_size {
            let new_active_buffer = Box::new(BTreeMap::new());
            let old_active_buffer = std::mem::replace(&mut self.active_buffer, new_active_buffer);
            self.bg_buffer = Arc::new(old_active_buffer);
            let bg_buffer_copy = self.bg_buffer.clone();
            self.bg_flusher.set_bg_write_buffer(bg_buffer_copy);
        }
    }

    pub fn find_node(&self, idx: u32) -> Option<i32> {
        let node = self.active_buffer.get(&idx);
        if node.is_none() {
            let node = self.bg_buffer.get(&idx);
            if node.is_some() {
                return Some(node.unwrap().clone());
            } else {
                return None;
            }
        }
        Some(node.unwrap().clone())
    }
}
