use std::process::exit;

use rs_sqlite::{to_u8_array, Row, EXIT_SUCCESS, ROW_SIZE, TABLE_MAX_ROWS};
use scanf::sscanf;

use crate::pager::Pager;

enum ExecuteResult {
    TableFull,
    NoExecute,
    Success,
}
enum PrepareResult {
    Success,
    UnrecognizedStatement,
    SyntaxError,
    ParseStringTooLong,
    ParseNegativeId,
}

#[derive(Debug)]
enum StatementType {
    Select,
    Insert,
    Unknown,
}

#[derive(Debug)]
struct Statement {
    typ: StatementType,
    row: Option<Row>,
}

pub struct Table {
    num_rows: usize,
    pager: Pager,
}

impl Table {
    pub fn db_open(filename: &str) -> Table {
        let pager = Pager::open(filename).unwrap();
        let num_rows = pager.file_length / ROW_SIZE;
        Self { num_rows, pager }
    }

    fn insert(&mut self, statement: &Statement) -> ExecuteResult {
        if self.num_rows >= TABLE_MAX_ROWS {
            return ExecuteResult::TableFull;
        }

        let Some(row) = &statement.row else {
            return ExecuteResult::NoExecute;
        };

        let insert_data = row.encode();
        self.pager.save_row(self.num_rows, insert_data);

        self.num_rows += 1;
        ExecuteResult::Success
    }

    fn select(&mut self, _statement: &Statement) -> ExecuteResult {
        for i in 0..self.num_rows {
            let select_data = self.pager.get_row(i);
            let row = Row::from(&select_data).unwrap();
            println!("{}", row);
        }

        ExecuteResult::Success
    }

    fn execute_statement(&mut self, statement: &Statement) -> ExecuteResult {
        match statement.typ {
            StatementType::Insert => self.insert(statement),
            StatementType::Select => self.select(statement),
            StatementType::Unknown => ExecuteResult::NoExecute,
        }
    }

    pub fn free(&mut self) {
        self.pager.free()
    }

    pub fn run(&mut self, src: &str) {
        if src.starts_with('.') {
            match src {
                ".exit" => {
                    self.free();
                    println!("Bye~");
                    exit(EXIT_SUCCESS);
                }
                _ => {
                    println!("Unrecognized command {}", src);
                }
            }
            return;
        }

        let mut statement = Statement {
            typ: StatementType::Unknown,
            row: None,
        };
        match prepare_statement(src, &mut statement) {
            PrepareResult::Success => match self.execute_statement(&statement) {
                ExecuteResult::Success => {
                    println!("Executed.");
                }
                ExecuteResult::TableFull => {
                    println!("Error: Table full.")
                }
                ExecuteResult::NoExecute => {}
            },
            PrepareResult::UnrecognizedStatement => {
                println!("Unrecognized keyword at start of '{}'.", src);
            }
            PrepareResult::SyntaxError => {
                println!("Syntax error. Could not parse statement.");
            }
            PrepareResult::ParseStringTooLong => {
                println!("String is too long.")
            }
            PrepareResult::ParseNegativeId => {
                println!("ID must be positive.")
            }
        }
    }
}

fn prepare_statement(src: &str, statement: &mut Statement) -> PrepareResult {
    if src.len() < 6 {
        return PrepareResult::UnrecognizedStatement;
    }

    match &src[0..6] {
        "insert" => {
            statement.typ = StatementType::Insert;
            let mut id: i32 = 0;
            let mut username: String = String::new();
            let mut email: String = String::new();
            if sscanf!(src, "insert {} {} {}", id, username, email).is_err() {
                return PrepareResult::SyntaxError;
            }

            if id < 0 {
                return PrepareResult::ParseNegativeId;
            }
            if username.len() > 32 || email.len() > 255 {
                return PrepareResult::ParseStringTooLong;
            }

            statement.row = Some(Row {
                id: id.try_into().unwrap(),
                username: to_u8_array::<32>(&username),
                email: to_u8_array::<255>(&email),
            });
        }
        "select" => statement.typ = StatementType::Select,
        _ => {
            return PrepareResult::UnrecognizedStatement;
        }
    }

    PrepareResult::Success
}
