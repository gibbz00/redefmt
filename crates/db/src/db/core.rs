use rusqlite_migration::Migrations;

#[allow(private_bounds)]
pub trait Db: private::Sealed {
    fn migrations() -> &'static Migrations<'static>;
}

mod private {
    use crate::*;

    pub(super) trait Sealed {}

    impl Sealed for MainDb {}
    impl Sealed for CrateDb {}
}
