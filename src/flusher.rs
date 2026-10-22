use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle},
};

use crate::storage_engine::StorageEngine;

type WriteBuffer = Arc<Box<BTreeMap<u32, i32>>>;

pub struct FlushWorker {
    sender: Option<Sender<WriteBuffer>>,
    worker_thread: Option<JoinHandle<()>>,
}

impl FlushWorker {
    pub fn new<T>(storage_engine: T) -> Self
    where
        T: StorageEngine + Send + 'static,
    {
        let (sender, receiver) = mpsc::channel::<WriteBuffer>();

        let worker_thread = thread::spawn(move || {
            FlushWorker::bg_flush_worker(storage_engine, receiver);
        });

        Self {
            sender: Some(sender),
            worker_thread: Some(worker_thread),
        }
    }

    pub fn set_bg_write_buffer(&self, bg_write_buffer: WriteBuffer) {
        self.sender
            .as_ref()
            .expect("flush worker is shutting down")
            .send(bg_write_buffer)
            .expect("flush worker has stopped");
    }

    fn bg_flush_worker<T>(mut storage_engine: T, receiver: Receiver<WriteBuffer>)
    where
        T: StorageEngine + Send + 'static,
    {
        // Block until a buffer arrives or all senders are dropped.
        while let Ok(bg_write_buffer) = receiver.recv() {
            for (&idx, &node_val) in bg_write_buffer.iter() {
                println!("node {} {}", idx, node_val);
                storage_engine.write_node(idx, node_val).unwrap();
            }
        }
    }
}

impl Drop for FlushWorker {
    fn drop(&mut self) {
        // Close the channel so the worker exits after processing
        // any buffers already queued.
        self.sender.take();

        if let Some(worker_thread) = self.worker_thread.take() {
            worker_thread.join().expect("flush worker panicked");
        }
    }
}
