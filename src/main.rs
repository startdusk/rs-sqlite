use std::env::args;
use std::io::{stdin, stdout, BufRead, Write};
use std::process::exit;

mod pager;
mod table;
use table::Table;

use rs_sqlite::EXIT_FAILURE;

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        println!("Must supply a database filename.");
        exit(EXIT_FAILURE);
    }

    let filename = &args[1];

    let stdin = stdin();
    print_prompt();
    let _ = stdout().flush();
    let mut table = Table::db_open(filename);
    for line in stdin.lock().lines() {
        let Ok(line) = line else {
            break;
        };
        if line.is_empty() {
            break;
        }

        table.run(&line.trim().to_lowercase());

        print_prompt();
        let _ = stdout().flush();
    }
}

fn print_prompt() {
    print!("db > ");
}
