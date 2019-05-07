use clap::{Arg, App};
use dir_diff::*;
use std::path::Path;

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

    let res = diff(vec![a, b].into_iter()).unwrap();
    let res = res.diff_paths(false);
    for entry in res {
         println!("{:?}", entry.0);
         for diffs in &entry.1 {
              println!("{:?}: {:?}", diffs.0, diffs.1);
         }
    }
}