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

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    #[test]
    fn main_test() {
        use assert_cmd::Command;

        let db_name = "test.db";
        if Path::new(db_name).exists() {
            fs::remove_file(db_name).unwrap();
        }

        // first create database
        let bin_name = env!("CARGO_PKG_NAME");
        let mut cmd = Command::cargo_bin(bin_name).unwrap();
        let session = cmd.arg(db_name);
        session.assert().stdout("db > ");
        session
            .write_stdin("INSERT 1 abc abc@gmail.com")
            .assert()
            .stdout("db > Executed.\ndb > ");

        session
            .write_stdin("SELECT")
            .assert()
            .stdout("db > (1, abc, abc@gmail.com)\nExecuted.\ndb > ");
        session
            .write_stdin("INSERT 2 byd byd@gmail.com")
            .assert()
            .stdout("db > Executed.\ndb > ");
        session
            .write_stdin("SELECT")
            .assert()
            .stdout("db > (1, abc, abc@gmail.com)\n(2, byd, byd@gmail.com)\nExecuted.\ndb > ");
        session.write_stdin(".exit").assert().stdout("db > Bye~\n");

        // test database persistence
        let mut cmd = Command::cargo_bin(bin_name).unwrap();
        let session = cmd.arg(db_name);
        session.assert().stdout("db > ");
        session
            .write_stdin("SELECT")
            .assert()
            .stdout("db > (1, abc, abc@gmail.com)\n(2, byd, byd@gmail.com)\nExecuted.\ndb > ");
        session
            .write_stdin("INSERT 3 li li@gmail.com")
            .assert()
            .stdout("db > Executed.\ndb > ");
        session
            .write_stdin("SELECT")
            .assert().stdout("db > (1, abc, abc@gmail.com)\n(2, byd, byd@gmail.com)\n(3, li, li@gmail.com)\nExecuted.\ndb > ");
        session.write_stdin(".exit").assert().stdout("db > Bye~\n");
    }
}
