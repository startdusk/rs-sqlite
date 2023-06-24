use std::fs::File;
use std::io;
use std::os::unix::prelude::FileExt;

// 读取一个文件, 从偏移位置开始读取数据, 读够8个字节
fn main() -> io::Result<()> {
    let mut buf = [0u8; 8];
    let file = File::open("./foo.test.txt")?;

    // We now read exactly 8 bytes from the offset 10.
    file.read_exact_at(&mut buf, 10)?;
    println!(
        "read {} bytes: {:?}",
        buf.len(),
        String::from_utf8_lossy(&buf)
    );
    Ok(())
}