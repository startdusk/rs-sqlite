use binary_serde::{BinarySerde, Endianness};
use scanf::sscanf;
use std::io::{stdin, stdout, BufRead, Write};
use std::process::exit;

fn main() {
    let stdin = stdin();
    print_prompt();
    let _ = stdout().flush();
    let mut table = Table::new();
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

#[repr(C)]
#[derive(Debug, BinarySerde)]
struct Row {
    id: u32,
    username: Username,
    email: Email,
}

impl std::fmt::Display for Row {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.id, self.username, self.email)
    }
}

#[derive(Debug, BinarySerde, Clone, Eq, PartialEq, Hash)]
struct Username([u8; 32]);

impl std::fmt::Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            String::from_utf8(self.0.to_vec())
                .unwrap()
                .replace('\0', "")
        )
    }
}

impl From<String> for Username {
    fn from(s: String) -> Self {
        let mut arr = [0u8; 32];
        s.bytes().zip(arr.iter_mut()).for_each(|(b, ptr)| *ptr = b);
        Username(arr)
    }
}

#[derive(Debug, BinarySerde, Clone, Eq, PartialEq, Hash)]
struct Email([u8; 255]);

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            String::from_utf8(self.0.to_vec())
                .unwrap()
                .replace('\0', "")
        )
    }
}

impl From<String> for Email {
    fn from(s: String) -> Self {
        let mut arr = [0u8; 255];
        s.bytes().zip(arr.iter_mut()).for_each(|(b, ptr)| *ptr = b);
        Email(arr)
    }
}

#[derive(Debug)]
struct Statement {
    typ: StatementType,
    row: Option<Row>,
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
            if let Err(_) = sscanf!(src, "insert {} {} {}", id, username, email) {
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
                username: username.into(),
                email: email.into(),
            });
        }
        "select" => statement.typ = StatementType::Select,
        _ => {
            return PrepareResult::UnrecognizedStatement;
        }
    }

    PrepareResult::Success
}

fn print_prompt() {
    print!("db > ");
}

const PAGE_SIZE: usize = 4096;
const TABLE_MAX_PAGES: usize = 100;
// ROW_SIZE = 291
const ROW_SIZE: usize = <Row as BinarySerde>::MAX_SERIALIZED_SIZE;
// ROWS_PER_PAGE 每个页到底有多少行, 291是计算Row结构体的所有字段的总长度
const ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE;
// TABLE_MAX_ROWS 表最大行数
const TABLE_MAX_ROWS: usize = ROWS_PER_PAGE * TABLE_MAX_PAGES;

struct Table {
    num_rows: usize,
    pages: [Option<[u8; PAGE_SIZE]>; TABLE_MAX_PAGES],
}

impl Table {
    pub fn new() -> Self {
        Self {
            num_rows: 0,
            pages: [None; TABLE_MAX_PAGES],
        }
    }
    fn page_num(&self, row_num: usize) -> usize {
        row_num / ROWS_PER_PAGE
    }

    fn row_solt(&self, row_num: usize) -> usize {
        let row_offset = row_num % ROWS_PER_PAGE;
        let byte_offset = row_offset * ROW_SIZE;
        byte_offset
    }

    fn insert(&mut self, statement: &Statement) -> ExecuteResult {
        if self.num_rows >= TABLE_MAX_ROWS {
            return ExecuteResult::TableFull;
        }

        let Some(row) = &statement.row else {
            return ExecuteResult::NoExecute;
        };

        let mut insert_data = [0u8; ROW_SIZE];
        row.binary_serialize(&mut insert_data, Endianness::Big);
        let mut offset = self.row_solt(self.num_rows);
        let mut page = [0u8; PAGE_SIZE];
        let page_option = self.pages[self.page_num(self.num_rows)];
        if page_option.is_some() {
            page = page_option.unwrap();
        }
        for i in 0..insert_data.len() {
            page[offset] = insert_data[i];
            offset += 1;
        }
        self.pages[self.page_num(self.num_rows)] = Some(page);

        self.num_rows += 1;
        ExecuteResult::Success
    }

    fn select(&self, _statement: &Statement) -> ExecuteResult {
        for i in 0..self.num_rows {
            let page_option = self.pages[self.page_num(i)];
            if page_option.is_none() {
                continue;
            }
            let page = page_option.unwrap();
            let offset = self.row_solt(i);
            let select_data = page[offset..offset + ROW_SIZE].to_vec();
            let row = Row::binary_deserialize(&select_data, Endianness::Big).unwrap();
            println!("{}", row);
        }

        ExecuteResult::Success
    }

    pub fn execute_statement(&mut self, statement: &Statement) -> ExecuteResult {
        match statement.typ {
            StatementType::Insert => self.insert(statement),
            StatementType::Select => self.select(statement),
            StatementType::Unknown => ExecuteResult::NoExecute,
        }
    }

    pub fn free(&mut self) {
        for i in 0..self.pages.len() {
            self.pages[i] = None;
        }
    }

    pub fn run(&mut self, src: &str) {
        if src.starts_with(".") {
            match src {
                ".exit" => {
                    println!("Bye~");
                    exit(0);
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

enum ExecuteResult {
    TableFull,
    NoExecute,
    Success,
}
