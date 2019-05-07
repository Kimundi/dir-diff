use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::thread;
use std::io;
use std::fs;
use std::collections::HashSet;

pub struct Diff {
    trees: Vec<(PathBuf, HashMap<PathBuf, Entry>)>,
}

#[derive(Debug)]
pub enum Error {
    ThreadError
}

#[derive(Debug)]
pub enum Entry {
    Metadata(fs::Metadata),
    MetadataError(walkdir::Error),
    EntryError,
    EntryIoError(io::ErrorKind),
}

pub fn diff<'a>(paths: impl Iterator<Item=&'a Path>) -> Result<Diff, Error> {
    let mut guards = Vec::new();
    let mut ret = Diff {
        trees: Vec::new()
    };
    
    for path in paths {
        let path = path.to_owned();
        guards.push(thread::spawn(move || {
            let mut paths = HashMap::new();

            let walkdir = walkdir::WalkDir::new(&path);
            for entry in walkdir.into_iter() {
                let walk_path;
                let res_entry;
                
                match entry {
                    Ok(e) => {
                        walk_path = e.path().to_owned();
                        res_entry = match e.metadata() {
                            Ok(x) => Entry::Metadata(x),
                            Err(x) => Entry::MetadataError(x),
                        }
                    }
                    Err(e) => {
                        if let Some(p) = e.path() {
                            walk_path = p.to_owned();
                            res_entry = match e.io_error() {
                                Some(e) => Entry::EntryIoError(e.kind()),
                                None => Entry::EntryError,
                            };
                        } else {
                            continue;
                        }
                    }
                }

                let rel_path = walk_path.strip_prefix(&path).unwrap();

                paths.insert(rel_path.to_owned(), res_entry);
            }

            (path, paths)
        }));
    }
    for guard in guards {
        ret.trees.push(guard.join().map_err(|_| Error::ThreadError)?);
    }
    Ok(ret)
}

pub type DiffEntry<'a> = (&'a Path, Vec<(&'a Path, &'a Entry)>);
pub type DiffEntries<'a> = Vec<DiffEntry<'a>>;

impl Diff {
    pub fn diff_paths(&self, filter_dirs: bool) -> DiffEntries<'_> {
        let mut diff_list = Vec::new();
        let mut all_rel_paths = HashSet::new();
        let mut dir_filter_cache = HashSet::new();

        for tree in &self.trees {
            for key in tree.1.keys() {
                all_rel_paths.insert(key);
            }
        }

        for rel_path in all_rel_paths {
            if let Some(p) = rel_path.parent() {
                if filter_dirs && dir_filter_cache.contains(p) {
                    continue;
                }
            }
            
            let mut local_diff_list: DiffEntry = (&rel_path, Vec::new());
            let mut add_to_diff_list = false;
            let mut add_to_dir_filter = false;
            
            for tree in &self.trees {
                match tree.1.get(rel_path) {
                    Some(e) => {
                        local_diff_list.1.push((&tree.0, e));
                    }
                    None => {
                        add_to_diff_list = true;
                    }
                }
            }
            
            for pair in local_diff_list.1.windows(2) {
                let a = &pair[0];
                let b = &pair[1];
                match (a.1, b.1) {
                    (Entry::Metadata(a), Entry::Metadata(b)) => {
                        if a.file_type() != b.file_type() || a.len() != b.len() {
                            add_to_diff_list = true;
                        }
                        if a.file_type().is_dir() || b.file_type().is_dir() {
                            add_to_dir_filter = true;
                        }
                    }
                    (Entry::MetadataError(a), Entry::MetadataError(b)) => {
                        if a.io_error().map(|e| e.kind()) != b.io_error().map(|e| e.kind()) {
                            add_to_diff_list = true;
                        }
                    }
                    (Entry::EntryError, Entry::EntryError) => {
                        // same...
                    }
                    (Entry::EntryIoError(a), Entry::EntryIoError(b)) => {
                        if a != b {
                            add_to_diff_list = true;
                        }
                    }
                    _ => add_to_diff_list = true,
                }
            }

            if add_to_diff_list {
                if add_to_dir_filter {
                    dir_filter_cache.insert(local_diff_list.0);
                }
                diff_list.push(local_diff_list);
            }
        }

        diff_list
    }
}