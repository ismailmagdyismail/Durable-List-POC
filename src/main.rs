use Durable_LinkedList::{
    heap_storage::HeapStorage,
    list_chain_view_storage::{ListChainNode, ListChainViewStorage},
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
    chain_storage.append(&list).unwrap();
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
}
