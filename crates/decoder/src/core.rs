use redefmt_common::codec::{Header, Stamp};
use tokio_util::bytes::{Buf, BytesMut};

use crate::*;

pub struct RedefmtDecoder {
    header: Option<Header>,
    stamp: Option<Stamp>,
}

impl RedefmtDecoder {
    pub fn new() -> Self {
        Self { header: None, stamp: None }
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

        if header.contains(Header::STAMP) && self.stamp.is_none() {
            if src.len() < std::mem::size_of::<u64>() {
                return Ok(None);
            }

            self.stamp = Some(Stamp::new(src.get_u64()));
        }

        // frame = (content)*
        // content = <write_content_id>(<type_hint>[<length_hint>](<type_hint>{1,2})[<type_bytes>])*
        // header: u8
        // - pointer width
        // - stamp included
        // type hint u8: primitives (13), collection: (4), write_statement_id (1)
        // - of the primitives, only the string needs to send a length hint
        // - of the collections, all send a length hint, and a type new hint,
        // - the map collection sends an extra type hint
        // - write_statement_id will in turn tell if if is a write_statement or type_structure

        // TEMP: replace with Ok(Item)
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use tokio_util::{bytes::BufMut, codec::Decoder};

    use super::*;

    #[test]
    fn empty_after_new() {
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
        assert!(bytes.is_empty());

        assert_eq!(expected_header, decoder.header.unwrap());
    }

    #[test]
    fn empty_after_header() {
        let mut decoder = RedefmtDecoder::new();
        decoder.header = Some(Header::all());

        let item = decoder.decode(&mut BytesMut::new()).unwrap();
        assert!(item.is_none());
    }

    #[test]
    fn stamp() {
        let header = Header::STAMP;
        let stamp = Stamp::new(123);

        let mut decoder = RedefmtDecoder::new();

        let mut bytes = BytesMut::new();
        bytes.put_u8(header.bits());
        bytes.put_u64(*stamp.as_ref());

        assert!(decoder.stamp.is_none());

        decoder.decode(&mut bytes).unwrap();
        assert!(bytes.is_empty());

        assert_eq!(stamp, decoder.stamp.unwrap());
    }
}
