use rs_sqlite::{ROWS_PER_PAGE, ROW_SIZE};

pub struct Cursor {
    row_num: usize,
    end_of_table: bool, // Indicates a position one past the last element
}

impl Cursor {
    pub fn table_start(row_num: usize) -> Self {
        Self {
            row_num: 0,
            end_of_table: row_num == 0,
        }
    }

    pub fn table_end(row_num: usize) -> Self {
        Self {
            row_num,
            end_of_table: true,
        }
    }

    pub fn offset(&self) -> usize {
        let row_num = self.row_num;
        let row_offset = row_num % ROWS_PER_PAGE;
        row_offset * ROW_SIZE
    }

    pub fn advance(&mut self, num_rows: usize) {
        self.row_num += 1;
        if self.row_num >= num_rows {
            self.end_of_table = true;
        }
    }

    pub fn is_end(&self) -> bool {
        self.end_of_table
    }
}
