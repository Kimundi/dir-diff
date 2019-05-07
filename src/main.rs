use clap::{Arg, App};
use dir_diff::*;
use std::path::Path;
use std::collections::HashMap;

fn main() {
    let matches = App::new("dir-diff")
        .version("1.0")
        .arg(Arg::with_name("A")
            .help("Sets the input a file to use")
            .required(true)
            .index(1))
        .arg(Arg::with_name("B")
            .help("Sets the input b file to use")
            .required(true)
            .index(2))
        .get_matches();
    let a = Path::new(matches.value_of_os("A").unwrap());
    let b = Path::new(matches.value_of_os("B").unwrap());
    println!("Compare path {:?}", a);
    println!("Compare path {:?}", b);

    let mut back_map = HashMap::new();
    back_map.insert(&a, "A");
    back_map.insert(&b, "B");

    let res = diff(vec![a, b].into_iter()).unwrap();
    let res = res.diff_paths(true);
    for entry in res {
        print!("[{}]: ", entry.0.display());
        let mut first = true;
        for diffs in &entry.1 {
            if first {first = false; } else {print!(" | "); }
            if let Some(entry) = diffs.1 {
                print!("{}: {}", back_map[&diffs.0], fmt(entry));
            } else {
                print!("{}: <missing>", back_map[&diffs.0]);
            }
        }
        println!();
    }
}

fn fmt(entry: &Entry) -> String {
    let fty = |t: std::fs::FileType| {
        if t.is_dir() {
            format!("directory")
        } else if t.is_file() {
            format!("file")
        } else  {
            format!("special")
        }
    };
    
    match entry {
        Entry::EntryError => format!("<dir walk entry error>"),
        Entry::EntryIoError(e) => format!("<dir walk entry io error: {:?}>", e),
        Entry::Metadata(m) => format!("type: {}, len: {}", fty(m.file_type()), m.len()),
        Entry::MetadataError(e) => format!("<dir walk entry metadata error: {:?}>", e),
    }
}