use binary_serde::{BinarySerde, Endianness};
use core::num;
use scanf::sscanf;
use std::env::args;
use std::fs::File;
use std::io::{stdin, stdout, BufRead, Seek, Write};
use std::os::unix::prelude::FileExt;
use std::process::exit;

use rs_sqlite::{
    Row, EXIT_FAILURE, EXIT_SUCCESS, PAGE_SIZE, ROWS_PER_PAGE, ROW_SIZE, TABLE_MAX_PAGES,
    TABLE_MAX_ROWS,
};

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
    let mut table = Table::db_open(&filename);
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

struct Table {
    num_rows: usize,
    // pages: [Option<[u8; PAGE_SIZE]>; TABLE_MAX_PAGES],
    pager: Pager,
}

impl Table {
    pub fn db_open(filename: &str) -> Table {
        let pager = Pager::open(filename).unwrap();
        let num_rows = (pager.file_length as usize) / ROW_SIZE;
        Self { num_rows, pager }
    }

    fn insert(&mut self, statement: &Statement) -> ExecuteResult {
        if self.num_rows >= TABLE_MAX_ROWS {
            return ExecuteResult::TableFull;
        }

        let Some(row) = &statement.row else {
            return ExecuteResult::NoExecute;
        };

        let mut insert_data = [0u8; ROW_SIZE];
        row.binary_serialize(&mut insert_data, Endianness::Little);
        self.pager.save_row(self.num_rows, insert_data);

        self.num_rows += 1;
        ExecuteResult::Success
    }

    fn select(&mut self, _statement: &Statement) -> ExecuteResult {
        for i in 0..self.num_rows {
            let page = self.pager.get_page(i);
            let offset = self.pager.offset(i);
            let select_data = page[offset..offset + ROW_SIZE].to_vec();
            let row = Row::binary_deserialize(&select_data, Endianness::Little).unwrap();
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
        self.pager.free()
    }

    pub fn run(&mut self, src: &str) {
        if src.starts_with(".") {
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

enum ExecuteResult {
    TableFull,
    NoExecute,
    Success,
}

struct Pager {
    pub file_descriptor: File,
    pub file_length: usize,
    pub pages: [Option<Page>; TABLE_MAX_PAGES],
}

type Page = [u8; PAGE_SIZE];

impl Pager {
    pub fn open(filename: &str) -> std::io::Result<Self> {
        let file = File::options()
            .write(true)
            .create(true)
            .append(true)
            .read(true)
            .open(filename)?;

        let file_length = file.metadata().unwrap().len() as usize;
        Ok(Self {
            file_descriptor: file,
            file_length,
            pages: [None; TABLE_MAX_PAGES],
        })
    }

    pub fn save_row(&mut self, row_num: usize, row: [u8; ROW_SIZE]) {
        let page_num = self.page_num(row_num);
        if page_num > TABLE_MAX_PAGES {
            println!(
                "Tried to fetch page number out of bounds. {} > {}",
                page_num, TABLE_MAX_PAGES
            );
            exit(EXIT_FAILURE);
        }
        let mut offset = self.offset(row_num);
        self.file_descriptor.write_at(&row, offset as u64).unwrap();
        let mut page = self.get_page(row_num);
        for i in 0..row.len() {
            page[offset] = row[i];
            offset += 1;
        }
        self.pages[page_num] = Some(page);
    }

    pub fn get_page(&mut self, row_num: usize) -> Page {
        let page_num = self.page_num(row_num);
        if page_num > TABLE_MAX_PAGES {
            println!(
                "Tried to fetch page number out of bounds. {} > {}",
                page_num, TABLE_MAX_PAGES
            );
            exit(EXIT_FAILURE);
        }
        let Some(page) = self.pages[page_num] else {
            // Cache miss. Allocate memory and load from file.
            let mut num_pages = self.file_length / PAGE_SIZE;

            // We might save a partial page at the end of the file
            if (self.file_length % PAGE_SIZE) > 0 {
                num_pages += 1;
            }
            // 读取指定数据
            let mut page: Page = [0u8; PAGE_SIZE];
            if num_pages > 0 && page_num <= num_pages {
                let offset = (page_num * PAGE_SIZE) as u64;
                self.file_descriptor.read_at(&mut page,  offset).unwrap();
            }
            self.pages[page_num] = Some(page);
            return page
        };

        page
    }

    pub fn free(&mut self) {
        for i in 0..self.pages.len() {
            self.pages[i] = None;
        }
    }

    pub fn page_num(&self, row_num: usize) -> usize {
        row_num / ROWS_PER_PAGE
    }

    pub fn offset(&self, row_num: usize) -> usize {
        let row_offset = row_num % ROWS_PER_PAGE;
        let byte_offset = row_offset * ROW_SIZE;
        byte_offset
    }
}
