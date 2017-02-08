use codec::header::Header;
use codec::primitives::{CqlStringMultiMap, decode};

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        HeaderError(::codec::header::Error);
    }

    errors {
        Incomplete {
            description("Unsufficient bytes")
            display("Buffer contains unsufficient bytes")
        }
        ParserError {
            description("Error during parsing")
            display("Error during parsing")
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SupportedMessage<'a>(pub CqlStringMultiMap<'a>);

#[derive(Debug, PartialEq)]
pub enum Message<'a> {
    Supported(SupportedMessage<'a>),
}

#[derive(Debug, PartialEq)]
struct Frame<'a> {
    header: Header,
    body: Message<'a>,
}

impl<'a> CqlDecode<'a, SupportedMessage<'a>> for SupportedMessage<'a> {
    // TODO: figure out how that works with draining an EasyBuf to do zero-copy
    fn decode(buf: &'a [u8]) -> Result<Self> {
        use nom::IResult;

        match decode::string_multimap(buf) {
            IResult::Done(_, output) => Ok(SupportedMessage(output)),
            IResult::Error(_) => Err(ErrorKind::ParserError.into()),
            IResult::Incomplete(err) => {
                println!("err = {:?}", err);
                Err(ErrorKind::Incomplete.into())
            }
        }
    }
}

pub trait CqlDecode<'a, T> {
    fn decode(buf: &'a [u8]) -> Result<T>;
}


#[cfg(test)]
mod test {
    use codec::header::Header;
    use super::*;

    fn skip_header(b: &[u8]) -> &[u8] {
        &b[Header::encoded_len()..]
    }

    #[test]
    fn decode_supported_message() {
        let msg = include_bytes!("../../tests/fixtures/v3/responses/supported.msg");
        let res = SupportedMessage::decode(skip_header(&msg[..])).unwrap();
        println!("res = {:?}", res);

        // TODO: do actual asserts
    }
}
