use std::borrow::Cow;

error_chain! {
     errors {
        MaximumLengthExceeded(l: usize) {
          description("Too many characters in string")
          display("Expected not more than {} characters, got {}.", u16::max_value(), l)
        }
    }
 }

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CqlString<'a> {
    inner: Cow<'a, str>,
}

impl<'a> CqlString<'a> {
    pub fn try_from(s: &'a str) -> Result<CqlString<'a>> {
        match s.len() > u16::max_value() as usize {
            true => Err(ErrorKind::MaximumLengthExceeded(s.len()).into()),
            false => Ok(CqlString { inner: Cow::Borrowed(s) }),
        }
    }

    pub unsafe fn unchecked_from(s: &'a str) -> CqlString<'a> {
        CqlString { inner: Cow::Borrowed(s) }
    }

    pub fn len(&self) -> u16 {
        self.inner.as_ref().len() as u16
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_ref().as_bytes()
    }
}

impl<'a> AsRef<str> for CqlString<'a> {
    fn as_ref(&self) -> &str {
        self.inner.as_ref()
    }
}

pub mod decode {
    use super::CqlString;
    use nom::be_u16;

    named!(pub short(&[u8]) -> u16, call!(be_u16));
    named!(pub string(&[u8]) -> CqlString, do_parse!(
            s: short          >>
            str: take_str!(s) >>
            (unsafe { CqlString::unchecked_from(str) })
            )
    );
}

pub mod encode {
    use byteorder::{ByteOrder, BigEndian};
    use super::CqlString;

    pub fn short(v: u16) -> [u8; 2] {
        let mut bytes = [0u8; 2];
        BigEndian::write_u16(&mut bytes[..], v);
        bytes
    }

    pub fn string(s: &CqlString, buf: &mut Vec<u8>) {
        buf.extend(&short(s.len())[..]);
        buf.extend(s.as_bytes());
    }

    pub fn string_list(l: &[CqlString], buf: &mut Vec<u8>) {
        buf.extend(&short(l.len() as u16 /*TODO strlist*/)[..]);
        for s in l {
            string(s, buf);
        }
    }
}

#[cfg(test)]
mod test {
    use super::{encode, decode, CqlString};

    #[test]
    fn short() {
        let expected: u16 = 342;
        let buf = encode::short(expected);

        assert_finished_and_eq!(decode::short(&buf), expected);
    }

    #[test]
    fn string() {
        let s = CqlString::try_from("Hello üß").unwrap();
        let mut buf = Vec::new();
        encode::string(&s, &mut buf);

        assert_finished_and_eq!(decode::string(&buf), s);
    }

    #[test]
    fn string_list() {
        let sl: Vec<_> = vec!["a", "b"]
            .iter()
            .map(|&s| CqlString::try_from(s))
            .map(Result::unwrap)
            .collect();

        let mut buf = Vec::new();
        encode::string_list(&sl, &mut buf);
        assert_eq!(&buf, b"\x00\x02\x00\x01a\x00\x01b");
    }
}
