use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CqlString<'a> {
    inner: Cow<'a, str>,
}

impl<'a> CqlString<'a> {
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

impl<'a> From<&'a str> for CqlString<'a> {
    fn from(s: &'a str) -> Self {
        CqlString { inner: Cow::Borrowed(s) }
    }
}

pub mod decode {
    use super::CqlString;
    use nom::be_u16;

    named!(pub short(&[u8]) -> u16, call!(be_u16));
    named!(pub string(&[u8]) -> CqlString, do_parse!(
            s: short          >>
            str: take_str!(s) >>
            (CqlString::from(str))
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
        buf.extend(&short(s.len() as u16)[..]);
        buf.extend(s.as_bytes());
    }

    //    pub fn string_list(l: &[&str], buf: &mut Vec<u8>) {
    //        buf.extend(&short(s.len() as u16)[..]);
    //    }
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
        let s = CqlString::from("Hello üß");
        let mut buf = Vec::new();
        encode::string(&s, &mut buf);

        assert_finished_and_eq!(decode::string(&buf), s);
    }
}
