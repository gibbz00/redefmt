use std::path::{Path, PathBuf};

use crate::*;

pub enum DbType<'a> {
    Main,
    Crate(CrateName<'a>),
}

impl DbType<'_> {
    pub(crate) fn path(&self, dir: &Path) -> PathBuf {
        match self {
            DbType::Main => dir.join("main.sqlite"),
            DbType::Crate(crate_name) => dir.join("crates").join(crate_name.as_ref()).join("db.sqlite"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_path() {
        let expected = "/tmp/main.sqlite";

        let actual = DbType::Main.path(Path::new("/tmp"));
        assert_eq!(expected, actual.as_os_str());

        let actual = DbType::Main.path(Path::new("/tmp/"));
        assert_eq!(expected, actual.as_os_str());
    }

    #[test]
    fn crate_path() {
        let expected = "/tmp/crates/abc/db.sqlite";

        let path = Path::new("/tmp");
        let crate_name = CrateName::new("abc").unwrap();
        let actual = DbType::Crate(crate_name).path(path);

        assert_eq!(expected, actual.as_os_str());
    }
}
