use codec::header::Header;
use codec::primitives::CqlStringMultiMap;
use codec::primitives::decode;

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        HeaderError(::codec::header::Error);
        DecodeError(::codec::primitives::decode::Error);
    }

    errors {
        Incomplete(err: String) {
            description("Unsufficient bytes")
            display("Buffer contains unsufficient bytes: {}", err)
        }
        ParserError(err: String) {
            description("Error during parsing")
            display("{}", err)
        }
    }
}

#[derive(Debug)]
pub struct SupportedMessage(pub CqlStringMultiMap<::tokio_core::io::EasyBuf>);

#[derive(Debug)]
pub enum Message {
    Supported(SupportedMessage),
    Ready,
}

#[derive(Debug)]
struct _Frame {
    header: Header,
    body: Message,
}

impl CqlDecode<SupportedMessage> for SupportedMessage {
    fn decode(buf: ::tokio_core::io::EasyBuf) -> Result<SupportedMessage> {
        decode::string_multimap(buf)
            .map(|d| d.1.into())
            .map_err(|err| ErrorKind::ParserError(format!("{}", err)).into())
    }
}

impl From<CqlStringMultiMap<::tokio_core::io::EasyBuf>> for SupportedMessage {
    fn from(v: CqlStringMultiMap<::tokio_core::io::EasyBuf>) -> Self {
        SupportedMessage(v)
    }
}

pub trait CqlDecode<T> {
    fn decode(buf: ::tokio_core::io::EasyBuf) -> Result<T>;
}

#[cfg(test)]
mod test {
    use codec::header::Header;
    use codec::primitives::{CqlFrom, CqlStringMultiMap, CqlString, CqlStringList};
    use super::*;

    fn skip_header(b: &[u8]) -> &[u8] {
        &b[Header::encoded_len()..]
    }

    #[test]
    fn decode_supported_message() {
        let msg = include_bytes!("../../tests/fixtures/v3/responses/supported.msg");
        let buf = Vec::from(skip_header(&msg[..])).into();
        let res = SupportedMessage::decode(buf).unwrap();

        let sla = ["3.2.1"];
        let slb = ["snappy", "lz4"];
        let csl1 = CqlStringList::try_from_iter_easy(sla.iter().cloned()).unwrap();
        let csl2 = CqlStringList::try_from_iter_easy(slb.iter().cloned()).unwrap();
        let smm =
            CqlStringMultiMap::try_from_iter(vec![(CqlString::try_from("CQL_VERSION").unwrap(),
                                                   csl1),
                                                  (CqlString::try_from("COMPRESSION").unwrap(),
                                                   csl2)])
                .unwrap();

        // TODO: do real comparison like
        //        assert_eq!(res, SupportedMessage(smm));
        assert_eq!(format!("{:?}", res), format!("{:?}", SupportedMessage(smm)));
    }


}
