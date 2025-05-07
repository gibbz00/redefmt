use std::{
    collections::{HashMap, hash_map::Entry},
    path::PathBuf,
};

use redefmt_common::codec::{Header, Stamp};
use redefmt_db::{
    CrateDb, DbClient, DbClientError, MainDb,
    crate_table::{Crate, CrateDbClient, CrateId, CrateName},
    state_dir::{StateDir, StateDirError},
    statement_table::{
        StatementDbClient,
        print::{PrintStatement, PrintStatementId},
    },
};
use tokio_util::bytes::{Buf, BytesMut};

use crate::*;

#[derive(Debug, thiserror::Error)]
pub enum RedefmtDecoderError {
    #[error("state directory resolution error")]
    StateDir(#[from] StateDirError),
    #[error("database failure")]
    Db(#[from] DbClientError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("unknown bits present in header '{0:?}'")]
    UnknownHeader(u8),
    #[error("no crate with ID '{0}' registered")]
    UnknownCrate(CrateId),
    #[error("no print statement with ID '{0}' registered for '{1}'")]
    UnknownPrintStatement(PrintStatementId, CrateName<'static>),
}

pub struct RedefmtDecoder {
    state_dir: PathBuf,
    main_db: DbClient<MainDb>,

    // Caches of sorts
    // IMPROVEMENT: use a proper cache crate?
    crate_dbs: HashMap<CrateId, (DbClient<CrateDb>, Crate<'static>)>,
    print_staments: HashMap<PrintStatementId, PrintStatement<'static>>,

    /// stage 1:
    header: Option<Header>,
    /// stage 2:
    stamp: Option<Stamp>,
    /// stage 3:
    print_crate_id: Option<CrateId>,
    /// stage 4:
    print_statement_id: Option<PrintStatementId>,
}

impl RedefmtDecoder {
    pub fn new() -> Result<Self, RedefmtDecoderError> {
        let dir = StateDir::resolve()?;
        Self::new_impl(dir)
    }

    pub(crate) fn new_impl(db_dir: PathBuf) -> Result<Self, RedefmtDecoderError> {
        let main_db = DbClient::new_main(&db_dir)?;

        Ok(Self {
            state_dir: db_dir,
            main_db,
            crate_dbs: Default::default(),
            print_staments: Default::default(),
            header: None,
            stamp: None,
            print_crate_id: None,
            print_statement_id: None,
        })
    }

    fn cache_crate_db(&mut self, crate_id: CrateId) -> Result<(), RedefmtDecoderError> {
        if let Entry::Vacant(vacant_entry) = self.crate_dbs.entry(crate_id) {
            let Some(crate_record) = self.main_db.find_crate_by_id(crate_id)? else {
                return Err(RedefmtDecoderError::UnknownCrate(crate_id));
            };

            let crate_db = DbClient::new_crate(&self.state_dir, crate_record.name())?;

            vacant_entry.insert((crate_db, crate_record));
        }

        Ok(())
    }

    /// Panics if no crate DB is cached for the given `crate_id`
    fn cache_print_statement(
        &mut self,
        crate_id: CrateId,
        print_statement_id: PrintStatementId,
    ) -> Result<(), RedefmtDecoderError> {
        if let Entry::Vacant(entry) = self.print_staments.entry(print_statement_id) {
            let (print_crate_db, print_crate_record) =
                self.crate_dbs.get(&crate_id).expect("crate database not loaded");

            let Some(print_statement) = print_crate_db.find_statement_by_id::<PrintStatement>(print_statement_id)?
            else {
                return Err(RedefmtDecoderError::UnknownPrintStatement(
                    print_statement_id,
                    print_crate_record.name().clone(),
                ));
            };

            entry.insert(print_statement);
        }

        Ok(())
    }
}

impl tokio_util::codec::Decoder for RedefmtDecoder {
    type Error = RedefmtDecoderError;
    type Item = RedefmtFrame;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let header = match self.header {
            Some(header) => header,
            None => {
                if src.is_empty() {
                    return Ok(None);
                }

                let header_byte = src.get_u8();

                let header = Header::from_bits(header_byte).ok_or(RedefmtDecoderError::UnknownHeader(header_byte))?;

                self.header = Some(header);

                header
            }
        };

        // stamp
        {
            if header.contains(Header::STAMP) && self.stamp.is_none() {
                if src.len() < std::mem::size_of::<u64>() {
                    return Ok(None);
                }

                self.stamp = Some(Stamp::new(src.get_u64()));
            }
        }

        let crate_id = match self.print_crate_id {
            Some(crate_id) => crate_id,
            None => {
                if src.len() < std::mem::size_of::<CrateId>() {
                    return Ok(None);
                }

                let crate_id = CrateId::new(src.get_u16());

                self.print_crate_id = Some(crate_id);

                self.cache_crate_db(crate_id)?;

                crate_id
            }
        };

        let print_statement_id = match self.print_statement_id {
            Some(print_statement_id) => print_statement_id,
            None => {
                if src.len() < std::mem::size_of::<PrintStatementId>() {
                    return Ok(None);
                }

                let print_statement_id = PrintStatementId::new(src.get_u16());

                self.print_statement_id = Some(print_statement_id);

                self.cache_print_statement(crate_id, print_statement_id)?;

                print_statement_id
            }
        };

        // TEMP: replace with Ok(Item)
        Ok(None)
    }
}

#[cfg(test)]
mod mock {
    use tempfile::TempDir;

    use super::*;

    impl RedefmtDecoder {
        pub fn mock() -> (TempDir, Self) {
            let temp_dir = tempfile::tempdir().unwrap();

            let decoder = RedefmtDecoder::new_impl(temp_dir.path().to_path_buf()).unwrap();

            (temp_dir, decoder)
        }
    }
}

#[cfg(test)]
mod tests {
    use redefmt_args::FormatOptions;
    use redefmt_db::{
        crate_table::{Crate, CrateName},
        location,
        statement_table::{
            Segment,
            print::{LogLevel, PrintInfo},
        },
    };
    use tokio_util::{bytes::BufMut, codec::Decoder};

    use super::*;

    #[test]
    fn empty_after_new() {
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock();

        let item = decoder.decode(&mut BytesMut::new()).unwrap();
        assert!(item.is_none());
    }

    #[test]
    fn header() {
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock();

        let expected_header = Header::all();

        let mut bytes = BytesMut::from_iter([expected_header.bits()]);

        assert!(decoder.header.is_none());

        decoder.decode(&mut bytes).unwrap();
        assert!(bytes.is_empty());

        assert_eq!(expected_header, decoder.header.unwrap());
    }

    #[test]
    fn empty_after_header() {
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock();

        decoder.header = Some(Header::all());

        let item = decoder.decode(&mut BytesMut::new()).unwrap();
        assert!(item.is_none());
    }

    #[test]
    fn stamp() {
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock();

        let header = Header::STAMP;
        let stamp = Stamp::new(123);

        let mut bytes = BytesMut::new();
        bytes.put_u8(header.bits());
        bytes.put_u64(*stamp.as_ref());

        assert!(decoder.stamp.is_none());

        decoder.decode(&mut bytes).unwrap();
        assert!(bytes.is_empty());

        assert_eq!(stamp, decoder.stamp.unwrap());
    }

    #[test]
    fn empty_after_stamp() {
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock();

        decoder.header = Some(Header::new(false));

        let item = decoder.decode(&mut BytesMut::new()).unwrap();
        assert!(item.is_none());
    }

    #[test]
    fn print_crate_id() {
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock();

        decoder.header = Some(Header::new(false));

        let crate_id = mock_crate(&mut decoder);

        assert!(decoder.print_crate_id.is_none());
        assert!(decoder.crate_dbs.is_empty());

        decode_print_crate_id(&mut decoder, crate_id);

        assert!(decoder.print_crate_id.is_some_and(|id| id == crate_id));
        assert!(decoder.crate_dbs.contains_key(&crate_id));
    }

    #[test]
    fn print_crate_id_not_found_error() {
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock();

        decoder.header = Some(Header::new(false));

        let crate_id = CrateId::new(123);

        let mut bytes = BytesMut::new();
        bytes.put_u16(*crate_id.as_ref());

        let error = decoder.decode(&mut bytes).unwrap_err();
        assert!(matches!(error, RedefmtDecoderError::UnknownCrate(_)));
    }

    #[test]
    fn empty_after_print_crate_id() {
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock();

        decoder.header = Some(Header::new(false));
        decoder.print_crate_id = Some(CrateId::new(123));

        let item = decoder.decode(&mut BytesMut::new()).unwrap();
        assert!(item.is_none());
    }

    #[test]
    fn print_statement_id() {
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock();

        decoder.header = Some(Header::new(false));

        let crate_id = mock_crate(&mut decoder);
        decode_print_crate_id(&mut decoder, crate_id);

        let print_statement_id = mock_print_statement(&mut decoder);

        assert!(decoder.print_statement_id.is_none());
        assert!(decoder.print_staments.is_empty());

        decode_print_statement_id(&mut decoder, print_statement_id);

        assert!(decoder.print_statement_id.is_some_and(|id| id == print_statement_id));
        assert!(decoder.print_staments.contains_key(&print_statement_id));
    }

    #[test]
    fn print_statement_id_not_found_error() {
        let (_dir_guard, mut decoder) = RedefmtDecoder::mock();

        decoder.header = Some(Header::new(false));

        let crate_id = mock_crate(&mut decoder);
        decode_print_crate_id(&mut decoder, crate_id);

        let print_statement_id = PrintStatementId::new(123);

        let mut bytes = BytesMut::new();
        bytes.put_u16(*print_statement_id.as_ref());

        let error = decoder.decode(&mut bytes).unwrap_err();
        assert!(matches!(error, RedefmtDecoderError::UnknownPrintStatement(_, _)));
    }

    fn mock_crate(decoder: &mut RedefmtDecoder) -> CrateId {
        let crate_name = CrateName::new("x").unwrap();
        let crate_record = Crate::new(crate_name);
        decoder.main_db.insert_crate_record(&crate_record).unwrap()
    }

    fn mock_print_statement(decoder: &mut RedefmtDecoder) -> PrintStatementId {
        let crate_id = decoder.print_crate_id.unwrap();
        let (crate_db, _) = decoder.crate_dbs.get(&crate_id).unwrap();

        let print_info = PrintInfo::new(Some(LogLevel::Debug), location!());
        let print_statement = PrintStatement::new(
            print_info,
            vec![Segment::Str("x".into()), Segment::Arg(FormatOptions::default())],
        );

        crate_db.insert_statement(&print_statement).unwrap()
    }

    fn decode_print_crate_id(decoder: &mut RedefmtDecoder, crate_id: CrateId) {
        let mut bytes = BytesMut::new();
        bytes.put_u16(*crate_id.as_ref());
        decoder.decode(&mut bytes).unwrap();
    }

    fn decode_print_statement_id(decoder: &mut RedefmtDecoder, print_stament_id: PrintStatementId) {
        let mut bytes = BytesMut::new();
        bytes.put_u16(*print_stament_id.as_ref());
        decoder.decode(&mut bytes).unwrap();
    }
}
