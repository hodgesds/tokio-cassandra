use std::fmt::{Formatter, Debug};
use super::CqlFrom;
use tokio_core::io::EasyBuf;
use std::hash::{Hasher, Hash};

#[derive(Clone, PartialEq, Eq)]
pub struct CqlString<T>
    where T: AsRef<[u8]>
{
    buf: T,
}

impl<T> Debug for CqlString<T>
    where T: AsRef<[u8]>
{
    fn fmt(&self, f: &mut Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
        self.as_ref().fmt(f)
    }
}

impl<T> AsRef<str> for CqlString<T>
    where T: AsRef<[u8]>
{
    fn as_ref(&self) -> &str {
        ::std::str::from_utf8(&self.buf.as_ref()).unwrap()
    }
}

impl<T> Into<String> for CqlString<T>
    where T: AsRef<[u8]>
{
    fn into(self) -> String {
        String::from(self.as_ref())
    }
}


impl<T> Hash for CqlString<T>
    where T: AsRef<[u8]>
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}


impl CqlString<::tokio_core::io::EasyBuf> {
    pub fn from(buf: ::tokio_core::io::EasyBuf) -> CqlString<::tokio_core::io::EasyBuf> {
        CqlString { buf: buf }
    }
}

impl<'a> CqlFrom<CqlString<EasyBuf>, &'a str> for CqlString<EasyBuf> {
    unsafe fn unchecked_from(s: &str) -> CqlString<EasyBuf> {
        let vec = Vec::from(s);
        CqlString { buf: vec.into() }
    }

    fn max_len() -> usize {
        u16::max_value() as usize
    }
}

impl<'a> CqlFrom<CqlString<Vec<u8>>, &'a str> for CqlString<Vec<u8>> {
    unsafe fn unchecked_from(s: &str) -> CqlString<Vec<u8>> {
        let vec = Vec::from(s);
        CqlString { buf: vec }
    }

    fn max_len() -> usize {
        u16::max_value() as usize
    }
}

impl<T> CqlString<T>
    where T: AsRef<[u8]>
{
    pub fn len(&self) -> u16 {
        self.buf.as_ref().len() as u16
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.buf.as_ref()
    }
}

impl From<CqlString<EasyBuf>> for CqlString<Vec<u8>> {
    fn from(string: CqlString<EasyBuf>) -> CqlString<Vec<u8>> {
        CqlString { buf: string.buf.into() }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio_core::io::EasyBuf;
    use super::super::super::{encode, decode};

    #[test]
    fn from_easybuf_into_vec() {
        let a: CqlString<EasyBuf> = unsafe { CqlString::unchecked_from("AbC") };
        let b: CqlString<Vec<u8>> = a.into();
        assert_eq!("AbC", b.as_ref());
    }

    #[test]
    fn string() {
        let s = CqlString::try_from("Hello üß").unwrap();
        let mut buf = Vec::new();
        encode::string(&s, &mut buf);

        let buf = Vec::from(&buf[..]).into();

        println!("buf = {:?}", buf);
        let res = decode::string(buf);
        assert_eq!(res.unwrap().1, s);
    }
}
