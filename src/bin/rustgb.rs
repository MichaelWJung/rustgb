extern crate rustgb;

use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} ROM_FILE", args[0]);
        ::std::process::exit(1);
    }
    let path = Path::new(&args[1]);
    let mut file = File::open(path).expect(&format!("Error opening file: {}", args[1]));
    rustgb::run(&mut file);
}
