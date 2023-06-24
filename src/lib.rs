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

impl std::fmt::Display for Row {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.id, self.username, self.email)
    }
}

#[derive(Debug, BinarySerde, Clone, Eq, PartialEq, Hash)]
pub struct Username([u8; 32]);

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
pub struct Email([u8; 255]);

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
