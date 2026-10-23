use std::path::PathBuf;

use Durable_LinkedList::immutable_storage_engine::ImmutableStorageEngine;

fn main() {
    let mut storage_engine = ImmutableStorageEngine::new(PathBuf::from("storage_engine")).unwrap();
    storage_engine.write_node(0, 1).unwrap();
    storage_engine.write_node(1, 2).unwrap();

    println!("node 0 {}", storage_engine.read_node(0).unwrap().unwrap());
    println!("node 1 {}", storage_engine.read_node(1).unwrap().unwrap());

    println!("nodes count {}", storage_engine.get_nodes_count());
}
