use redefmt_common::codec::Header;
use tokio_util::bytes::{Buf, BytesMut};

use crate::*;

pub struct RedefmtDecoder {
    header: Option<Header>,
}

impl RedefmtDecoder {
    pub fn new() -> Self {
        Self { header: None }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RedefmtDecoderError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("unknown bits present in header '{0:?}'")]
    UnknownHeader(u8),
}

impl tokio_util::codec::Decoder for RedefmtDecoder {
    type Error = RedefmtDecoderError;
    type Item = RedefmtFrame;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let Some(header) = &self.header else {
            match src.is_empty() {
                true => return Ok(None),
                false => {
                    let header_byte = src.get_u8();

                    self.header = Header::from_bits(header_byte)
                        .map(Some)
                        .ok_or(RedefmtDecoderError::UnknownHeader(header_byte))?;

                    return Ok(None);
                }
            }
        };
        // frame = <header>[<stamp>](content)*
        // content = <write_content_id>(<type_hint>[<length_hint>](<type_hint>{1,2})[<type_bytes>])*
        // header: u8
        // - pointer width
        // - stamp included
        // type hint u8: primitives (13), collection: (4), write_statement_id (1)
        // - of the primitives, only the string needs to send a length hint
        // - of the collections, all send a length hint, and a type new hint,
        // - the map collection sends an extra type hint
        // - write_statement_id will in turn tell if if is a write_statement or type_structure

        todo!()
    }
}

#[cfg(test)]
mod tests {
    use tokio_util::codec::Decoder;

    use super::*;

    #[test]
    fn empty() {
        let mut decoder = RedefmtDecoder::new();

        let item = decoder.decode(&mut BytesMut::new()).unwrap();

        assert!(item.is_none());
    }

    #[test]
    fn header() {
        let expected_header = Header::all();

        let mut decoder = RedefmtDecoder::new();

        let mut bytes = BytesMut::from_iter([expected_header.bits()]);

        assert!(decoder.header.is_none());

        decoder.decode(&mut bytes).unwrap();

        assert_eq!(expected_header, decoder.header.unwrap());
        assert!(bytes.is_empty());
    }
}
