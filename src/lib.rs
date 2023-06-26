use binary_serde::BinarySerde;

pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_FAILURE: i32 = -1;
pub const PAGE_SIZE: usize = 4096;
pub const TABLE_MAX_PAGES: usize = 100;
// ROW_SIZE = 291
pub const ROW_SIZE: usize = <Row as BinarySerde>::MAX_SERIALIZED_SIZE;
// ROWS_PER_PAGE 每个页到底有多少行, 291是计算Row结构体的所有字段的总长度
pub const ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE;
// TABLE_MAX_ROWS 表最大行数
pub const TABLE_MAX_ROWS: usize = ROWS_PER_PAGE * TABLE_MAX_PAGES;

#[repr(C)]
#[derive(Debug, BinarySerde)]
pub struct Row {
    pub id: u32,
    pub username: Username,
    pub email: Email,
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
