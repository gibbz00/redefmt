use redefmt_db::{DbClient, MainDb};

pub struct DbClientProxy {
    main: DbClient<MainDb>,
}
