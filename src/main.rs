use std::{
    collections::BTreeMap,
    io::{self, BufRead, Write},
    path::PathBuf,
};

use Durable_LinkedList::immutable_storage_engine::ImmutableStorageEngine;

enum Command {
    Get(u32),
    Push(u32, i32),
    Commit,
    Abort,
    Help,
    Exit,
}

fn parse_command(line: &str) -> Result<Option<Command>, String> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    match parts.as_slice() {
        [] => Ok(None),
        ["get", idx] => idx
            .parse()
            .map(Command::Get)
            .map(Some)
            .map_err(|_| "get requires a valid u32 node index".to_string()),
        ["push", idx, value] => {
            let idx = idx
                .parse()
                .map_err(|_| "push requires a valid u32 node index".to_string())?;
            let value = value
                .parse()
                .map_err(|_| "push requires a valid i32 node value".to_string())?;
            Ok(Some(Command::Push(idx, value)))
        }
        ["commit"] => Ok(Some(Command::Commit)),
        ["abort"] => Ok(Some(Command::Abort)),
        ["help"] => Ok(Some(Command::Help)),
        ["exit"] | ["quit"] => Ok(Some(Command::Exit)),
        ["get", ..] => Err("usage: get <index>".to_string()),
        ["push", ..] => Err("usage: push <index> <value>".to_string()),
        _ => Err("unknown command; type help for available commands".to_string()),
    }
}

fn print_help() {
    println!("get <index>         read a node, including uncommitted changes");
    println!("push <index> <value> buffer a node write");
    println!("commit              write all buffered changes to storage");
    println!("abort               discard all buffered changes");
    println!("exit | quit         exit the REPL");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut storage_engine = ImmutableStorageEngine::new(PathBuf::from("storage_engine"))?;
    let mut pending_writes = BTreeMap::<u32, i32>::new();
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    println!("Durable LinkedList REPL; type help for available commands");

    loop {
        print!("> ");
        io::stdout().flush()?;

        let Some(line) = lines.next() else {
            println!();
            break;
        };
        let line = line?;

        let command = match parse_command(&line) {
            Ok(Some(command)) => command,
            Ok(None) => continue,
            Err(message) => {
                eprintln!("error: {message}");
                continue;
            }
        };

        match command {
            Command::Get(idx) => {
                if let Some(value) = pending_writes.get(&idx) {
                    println!("{value} (uncommitted)");
                    continue;
                }

                match storage_engine.read_node(idx) {
                    Ok(Some(value)) => println!("{value}"),
                    Ok(None) => println!("node {idx} not found"),
                    Err(error) => eprintln!("error reading node {idx}: {error}"),
                }
            }
            Command::Push(idx, value) => {
                pending_writes.insert(idx, value);
                println!("buffered node {idx} = {value}");
            }
            Command::Commit => {
                let nodes: Vec<(u32, i32)> = pending_writes
                    .iter()
                    .map(|(&idx, &value)| (idx, value))
                    .collect();

                match storage_engine.write_nodes_batch(&nodes) {
                    Ok(()) => {
                        let committed_count = nodes.len();
                        pending_writes.clear();
                        println!("committed {committed_count} node(s)");
                    }
                    Err(error) => {
                        eprintln!("commit failed: {error}; buffered changes were kept");
                    }
                }
            }
            Command::Abort => {
                let aborted_count = pending_writes.len();
                pending_writes.clear();
                println!("aborted {aborted_count} buffered node(s)");
            }
            Command::Help => print_help(),
            Command::Exit => break,
        }
    }

    Ok(())
}
