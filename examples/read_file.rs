use std::fs::File;
use std::io;
#[cfg(target_family = "unix")]
use std::os::unix::prelude::FileExt;
#[cfg(target_family = "windows")]
use std::os::windws::prelude::FileExt;

// 读取一个文件, 从偏移位置开始读取数据, 读够8个字节
fn main() -> io::Result<()> {
    let mut buf = [0u8; 8];
    let file = File::open("./foo.test.txt")?;

    // We now read exactly 8 bytes from the offset 10.
    #[cfg(target_family = "unix")]
    file.read_at(&mut buf, 10)?;
    #[cfg(target_family = "windows")]
    file.seek_read(&mut buf, 10)?;

    println!(
        "read {} bytes: {:?}",
        buf.len(),
        String::from_utf8_lossy(&buf)
    );
    Ok(())
}
