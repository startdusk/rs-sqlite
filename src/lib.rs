use binary_serde::{BinarySerde, DeserializeError, Endianness};

pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_FAILURE: i32 = -1;
pub const PAGE_SIZE: usize = 4096;

// windows 的栈要比 unix 的要小, 数组是在栈上分配, 改为vec在堆上分配
pub const TABLE_MAX_PAGES: usize = 100;

// ROW_SIZE = 291
pub const ROW_SIZE: usize = <Row as BinarySerde>::MAX_SERIALIZED_SIZE;
// ROWS_PER_PAGE 每个页到底有多少行, 291是计算Row结构体的所有字段的总长度
pub const ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE;
// TABLE_MAX_ROWS 表最大行数
pub const TABLE_MAX_ROWS: usize = ROWS_PER_PAGE * TABLE_MAX_PAGES;

pub type RowLine = [u8; ROW_SIZE];
pub type Page = [u8; PAGE_SIZE];

#[repr(C)]
#[derive(Debug, BinarySerde)]
pub struct Row {
    pub id: u32,
    pub username: Username,
    pub email: Email,
}

impl Row {
    pub fn encode(&self) -> RowLine {
        let mut insert_data: RowLine = [0u8; ROW_SIZE];
        self.binary_serialize(&mut insert_data, Endianness::Little);
        insert_data
    }

    pub fn from(buf: &[u8]) -> Result<Row, DeserializeError> {
        Row::binary_deserialize(buf, Endianness::Little)
    }
}

pub type Username = [u8; 32];

pub type Email = [u8; 255];

impl std::fmt::Display for Row {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let username = String::from_utf8(self.username.to_vec())
            .unwrap()
            .replace('\0', "");
        let email = String::from_utf8(self.email.to_vec())
            .unwrap()
            .replace('\0', "");
        write!(f, "({}, {}, {})", self.id, username, email)
    }
}

pub fn to_u8_array<const N: usize>(s: &str) -> [u8; N] {
    let mut a = [0u8; N];
    s.bytes().zip(a.iter_mut()).for_each(|(b, ptr)| *ptr = b);
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_u8_array_should_work() {
        assert_eq!(to_u8_array::<10>("0000000000"), [48u8; 10])
    }
}
