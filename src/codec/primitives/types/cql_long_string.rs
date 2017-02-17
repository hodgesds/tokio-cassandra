use std::fmt::{Formatter, Debug};
use super::CqlFrom;
use tokio_core::io::EasyBuf;
use std::hash::{Hasher, Hash};

#[derive(Clone, PartialEq, Eq)]
pub struct CqlLongString<T>
    where T: AsRef<[u8]>
{
    buf: T,
}

impl<T> Debug for CqlLongString<T>
    where T: AsRef<[u8]>
{
    fn fmt(&self, f: &mut Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
        self.as_ref().fmt(f)
    }
}

impl<T> AsRef<str> for CqlLongString<T>
    where T: AsRef<[u8]>
{
    fn as_ref(&self) -> &str {
        ::std::str::from_utf8(&self.buf.as_ref()).unwrap()
    }
}


impl<T> Hash for CqlLongString<T>
    where T: AsRef<[u8]>
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}


impl CqlLongString<::tokio_core::io::EasyBuf> {
    pub fn from(buf: ::tokio_core::io::EasyBuf) -> CqlLongString<::tokio_core::io::EasyBuf> {
        CqlLongString { buf: buf }
    }
}

impl<'a> CqlFrom<CqlLongString<EasyBuf>, &'a str> for CqlLongString<EasyBuf> {
    unsafe fn unchecked_from(s: &str) -> CqlLongString<EasyBuf> {
        let vec = Vec::from(s);
        CqlLongString { buf: vec.into() }
    }

    fn max_len() -> usize {
        i32::max_value() as usize
    }
}

impl<'a> CqlFrom<CqlLongString<Vec<u8>>, &'a str> for CqlLongString<Vec<u8>> {
    unsafe fn unchecked_from(s: &str) -> CqlLongString<Vec<u8>> {
        let vec = Vec::from(s);
        CqlLongString { buf: vec }
    }

    fn max_len() -> usize {
        i32::max_value() as usize
    }
}

impl<T> CqlLongString<T>
    where T: AsRef<[u8]>
{
    pub fn len(&self) -> i32 {
        self.buf.as_ref().len() as i32
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.buf.as_ref()
    }
}

impl From<CqlLongString<EasyBuf>> for CqlLongString<Vec<u8>> {
    fn from(string: CqlLongString<EasyBuf>) -> CqlLongString<Vec<u8>> {
        CqlLongString { buf: string.buf.into() }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio_core::io::EasyBuf;
    use super::super::super::{encode, decode};

    #[test]
    fn from_easybuf_into_vec() {
        let a: CqlLongString<EasyBuf> = unsafe { CqlLongString::unchecked_from("AbC") };
        let b: CqlLongString<Vec<u8>> = a.into();
        assert_eq!("AbC", b.as_ref());
    }

    #[test]
    fn string() {
        let s = CqlLongString::try_from("Hello üß in a long String").unwrap();
        let mut buf = Vec::new();
        encode::long_string(&s, &mut buf);

        let buf = Vec::from(&buf[..]).into();

        println!("buf = {:?}", buf);
        let res = decode::long_string(buf);
        assert_eq!(res.unwrap().1, s);
    }
}
