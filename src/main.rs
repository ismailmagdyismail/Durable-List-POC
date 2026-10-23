use std::{path::PathBuf, thread::sleep, time::Duration};

use Durable_LinkedList::{
    flusher::FlushWorker,
    heap_storage::HeapStorage,
    list_chain_view_storage::{ListChainNode, ListChainViewStorage},
    storage_engine::MockLogFileStorageEngine,
    write_buffer::WriteBuffer,
};

fn main() {
    std::fs::create_dir_all("durable_list").unwrap();
    let mut storage =
        HeapStorage::open(HeapStorage::create_heap_file_path("durable_list")).unwrap();
    let offset = storage.append(&[1, 2, 3, 4]).unwrap();
    storage.sync().unwrap();
    println!("offset is {}", offset);

    let mut cursor = storage.cursor().unwrap();
    while let Ok(res) = cursor.next_record() {
        if let Some(record) = res {
            println!("record is {}", record)
        } else {
            println!("end");
            break;
        }
    }

    let mut chain_storage = ListChainViewStorage::open(
        ListChainViewStorage::create_list_chain_file_path("durable_list"),
    )
    .unwrap();

    let list = [ListChainNode {
        prev_offset: 0,
        node_offset: 0,
        next_offset: 4,
    }];
    chain_storage.insert_node(0, list[0].node_offset).unwrap();
    chain_storage.sync().unwrap();
    let mut cursor = chain_storage.cursor().unwrap();
    while let Ok(res) = cursor.next_record() {
        if let Some(record) = res {
            println!("prev_offset is {}", record.prev_offset);
            println!("node_offset is {}", record.node_offset);
            println!("next_offset is {}", record.next_offset);
        } else {
            println!("end");
            break;
        }
    }

    let mut path = PathBuf::new();
    path.push("memtable.dat");
    let storage = MockLogFileStorageEngine::new(path).unwrap();
    let flusher = FlushWorker::new(storage);
    let mut memtable = WriteBuffer::new(10, flusher);
    for i in 0..10 {
        println!("insert node {} , {} ", i, i + 1);
        memtable.insert_node(i, (i + 1) as i32);
    }

    sleep(Duration::from_millis(10));
}
