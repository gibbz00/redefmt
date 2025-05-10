mod short_id;
pub(crate) use short_id::{ShortId, short_id_newtype};

short_id_newtype!(CrateId);

short_id_newtype!(PrintStatementId);

short_id_newtype!(WriteStatementId);
