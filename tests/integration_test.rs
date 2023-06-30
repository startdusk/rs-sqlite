#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    struct DbFile {
        name: String,
    }

    impl DbFile {
        pub fn clean(db_name: &str) -> Self {
            if Path::new(db_name).exists() {
                fs::remove_file(db_name).unwrap();
            }
            Self {
                name: db_name.to_string(),
            }
        }
    }

    impl Drop for DbFile {
        fn drop(&mut self) {
            fs::remove_file(&self.name).unwrap();
        }
    }

    #[test]
    fn main_test() {
        use assert_cmd::Command;

        let db_name = "test.db";
        // 注意: 一定要提取变量( _ 只是忽略返回值), 才会执行 Drop trait
        let _db_file = DbFile::clean(db_name);

        // first create database
        let bin_name = env!("CARGO_PKG_NAME");
        let mut cmd = Command::cargo_bin(bin_name).unwrap();
        let session = cmd.arg(db_name);
        session.assert().stdout("db > ");
        session
            .write_stdin("INSERT 1 abc abc@gmail.com")
            .assert()
            .stdout("db > Executed.\ndb > ");

        session
            .write_stdin("SELECT")
            .assert()
            .stdout("db > (1, abc, abc@gmail.com)\nExecuted.\ndb > ");
        session
            .write_stdin("INSERT 2 byd byd@gmail.com")
            .assert()
            .stdout("db > Executed.\ndb > ");
        session
            .write_stdin("SELECT")
            .assert()
            .stdout("db > (1, abc, abc@gmail.com)\n(2, byd, byd@gmail.com)\nExecuted.\ndb > ");
        session.write_stdin(".exit").assert().stdout("db > Bye~\n");

        // test database persistence
        let mut cmd = Command::cargo_bin(bin_name).unwrap();
        let session = cmd.arg(db_name);
        session.assert().stdout("db > ");
        session
            .write_stdin("SELECT")
            .assert()
            .stdout("db > (1, abc, abc@gmail.com)\n(2, byd, byd@gmail.com)\nExecuted.\ndb > ");
        session
            .write_stdin("INSERT 3 li li@gmail.com")
            .assert()
            .stdout("db > Executed.\ndb > ");
        session
            .write_stdin("SELECT")
            .assert().stdout("db > (1, abc, abc@gmail.com)\n(2, byd, byd@gmail.com)\n(3, li, li@gmail.com)\nExecuted.\ndb > ");
        session.write_stdin(".exit").assert().stdout("db > Bye~\n");
    }
}
