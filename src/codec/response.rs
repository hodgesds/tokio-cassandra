use codec::header::Header;

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        HeaderError(::codec::header::Error);
    }
}

#[derive(Debug, PartialEq)]
pub struct SupportedMessage;

#[derive(Debug, PartialEq)]
pub enum Message {
    Supported(SupportedMessage),
}

#[derive(Debug, PartialEq)]
struct Frame {
    header: Header,
    body: Message,
}

impl CqlDecode<SupportedMessage> for SupportedMessage {
    fn decode(_buf: &[u8]) -> Result<Self> {
        unimplemented!()
    }
}

pub trait CqlDecode<T> {
    fn decode(buf: &[u8]) -> Result<T>;
}


#[cfg(test)]
mod test {
    use codec::header::Header;
    use super::*;

    fn skip_header(b: &[u8]) -> &[u8] {
        &b[..Header::encoded_len()]
    }

    #[test]
    fn decode_supported_message() {
        let msg = include_bytes!("../../tests/fixtures/v3/responses/supported.msg");
        let res = SupportedMessage::decode(skip_header(&msg[..])).unwrap();
        assert_eq!(res, SupportedMessage);
    }
}
