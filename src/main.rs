use Durable_LinkedList::{heap_cursor::HeapCursor, heap_storage::HeapStorage};

fn main() {
    std::fs::create_dir_all("durable_list").unwrap();
    let mut storage = HeapStorage::new("durable_list/").unwrap();
    let offset = storage.append_records_batch(&[1, 2, 3, 4]).unwrap();
    storage.sync().unwrap();
    println!("offset is {}", offset);

    let mut cursor = HeapCursor::new("durable_list").unwrap();
    while let Ok(res) = cursor.next_record() {
        if let Some(record) = res {
            println!("record is {}", record)
        } else {
            println!("end");
            break;
        }
    }
}
