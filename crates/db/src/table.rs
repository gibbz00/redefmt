use crate::*;

pub trait Record {
    type Id;
}

pub trait Table<R: Record> {
    fn find_by_id(&self, id: R::Id) -> Result<Option<R>, DbClientError>;

    fn insert(&self, record: &R) -> Result<R::Id, DbClientError>;
}
