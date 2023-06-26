use std::{fs::File, process::exit};

#[cfg(target_family = "unix")]
use std::os::unix::prelude::FileExt;
#[cfg(target_family = "windows")]
use std::os::windws::prelude::FileExt;

use rs_sqlite::{EXIT_FAILURE, PAGE_SIZE, ROWS_PER_PAGE, ROW_SIZE, TABLE_MAX_PAGES};

pub struct Pager {
    pub file_descriptor: File,
    pub file_length: usize,
    pub pages: [Option<Page>; TABLE_MAX_PAGES],
}

pub struct RowType([u8; ROW_SIZE]);

impl TryFrom<Vec<u8>> for RowType {
    type Error = String;

    fn try_from(v: Vec<u8>) -> Result<Self, Self::Error> {
        let slice = v.as_slice();
        let array: [u8; ROW_SIZE] = match slice.try_into() {
            Ok(ba) => ba,
            Err(_) => panic!("Expected a Vec of length {} but it was {}", 32, v.len()),
        };
        Ok(RowType(array))
    }
}

pub type Page = [u8; PAGE_SIZE];

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
        let offset = self.offset(row_num);

        #[cfg(target_family = "unix")]
        self.file_descriptor.write_at(&row, offset as u64).unwrap();

        #[cfg(target_family = "windows")]
        self.file_descriptor
            .seek_write(&row, offset as u64)
            .unwrap();

        let mut page = self.get_page(row_num);
        page[offset..(row.len() + offset)].copy_from_slice(&row[..]);
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
                #[cfg(target_family = "unix")]
                self.file_descriptor.read_at(&mut page,  offset).unwrap();
                #[cfg(target_family = "windows")]
                self.file_descriptor.seek_read(&mut page,  offset).unwrap();
            }
            self.pages[page_num] = Some(page);
            return page
        };

        page
    }

    pub fn get_row(&mut self, row_num: usize) -> [u8; ROW_SIZE] {
        let page = self.get_page(row_num);
        let offset = self.offset(row_num);
        let select_data = page[offset..offset + ROW_SIZE].to_vec();
        let data_length = select_data.len();
        let array: [u8; ROW_SIZE] = match select_data.try_into() {
            Ok(ba) => ba,
            Err(_) => panic!(
                "Expected a data of length {} but it was {}",
                ROW_SIZE, data_length,
            ),
        };
        array
    }

    pub fn free(&mut self) {
        for i in 0..self.pages.len() {
            self.pages[i] = None;
        }
    }

    fn page_num(&self, row_num: usize) -> usize {
        row_num / ROWS_PER_PAGE
    }

    fn offset(&self, row_num: usize) -> usize {
        let row_offset = row_num % ROWS_PER_PAGE;
        row_offset * ROW_SIZE
    }
}
