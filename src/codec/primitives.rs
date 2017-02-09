use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::{Hasher, Hash};
use std::convert::AsRef;

error_chain! {
     errors {
        MaximumLengthExceeded(l: usize) {
          description("Too many elements container")
          display("Expected not more than {} elements, got {}.", u16::max_value(), l)
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

impl<'a> Hash for CqlString<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state)
    }
}

/// TODO: zero copy - implement it as offset to beginning of vec to CqlStrings to prevent Vec
/// allocation
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CqlStringList<'a> {
    container: Vec<CqlString<'a>>,
}

impl<'a> CqlStringList<'a> {
    pub fn try_from(lst: Vec<CqlString<'a>>) -> Result<CqlStringList<'a>> {
        match lst.len() > u16::max_value() as usize {
            true => Err(ErrorKind::MaximumLengthExceeded(lst.len()).into()),
            false => Ok(CqlStringList { container: lst }),
        }
    }

    pub fn try_from_iter<I, E, S>(v: I) -> Result<CqlStringList<'a>>
        where I: IntoIterator<IntoIter = E, Item = S>,
              E: Iterator<Item = S> + ExactSizeIterator,
              S: AsRef<str> + 'a
    {
        let mut v = v.into_iter();
        let mut res = Vec::with_capacity(v.len());
        for s in v {
            res.push(CqlString::try_from(s.as_ref())?);
        }
        CqlStringList::try_from(res)
    }

    pub unsafe fn unchecked_from(lst: Vec<CqlString<'a>>) -> CqlStringList<'a> {
        CqlStringList { container: lst }
    }

    pub fn len(&self) -> u16 {
        self.container.len() as u16
    }

    pub fn iter(&'a self) -> ::std::slice::Iter<'a, CqlString<'a>> {
        self.container.iter()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CqlStringMultiMap<'a> {
    container: HashMap<CqlString<'a>, CqlStringList<'a>>,
}

impl<'a> CqlStringMultiMap<'a> {
    pub fn try_from(map: HashMap<CqlString<'a>, CqlStringList<'a>>)
                    -> Result<CqlStringMultiMap<'a>> {
        match map.len() > u16::max_value() as usize {
            true => Err(ErrorKind::MaximumLengthExceeded(map.len()).into()),
            false => Ok(CqlStringMultiMap { container: map }),
        }
    }

    pub unsafe fn unchecked_from(lst: HashMap<CqlString<'a>, CqlStringList<'a>>)
                                 -> CqlStringMultiMap<'a> {
        CqlStringMultiMap { container: lst }
    }

    pub fn len(&self) -> u16 {
        self.container.len() as u16
    }

    pub fn iter(&'a self)
                -> ::std::collections::hash_map::Iter<'a, CqlString<'a>, CqlStringList<'a>> {
        self.container.iter()
    }
}

pub mod decode {
    use super::{CqlStringList, CqlString, CqlStringMultiMap};
    use nom::be_u16;
    use std::collections::HashMap;

    named!(pub short(&[u8]) -> u16, call!(be_u16));
    named!(pub string(&[u8]) -> CqlString, do_parse!(
            s: short          >>
            str: take_str!(s) >>
            (unsafe { CqlString::unchecked_from(str) })
        )
    );
    named!(pub string_list(&[u8]) -> CqlStringList, do_parse!(
            l: short >>
            list: count!(string, l as usize) >>
            (unsafe { CqlStringList::unchecked_from(list) })
        )
    );
    named!(pub string_multimap(&[u8]) -> CqlStringMultiMap,
        do_parse!(
            l: short >>
            mm: fold_many_m_n!(l as usize, l as usize,
                do_parse!(
                    key: string >>
                    value: string_list >>
                    (key, value)
                ),
                HashMap::new(),
                | mut map: HashMap<_,_>, (k, v) | {
                    map.insert(k, v);
                    map
                }
            )
             >>
            (unsafe { CqlStringMultiMap::unchecked_from(mm) })
        )
    );
}

pub mod encode {
    use byteorder::{ByteOrder, BigEndian};
    use super::{CqlStringList, CqlString, CqlStringMultiMap};

    pub fn short(v: u16) -> [u8; 2] {
        let mut bytes = [0u8; 2];
        BigEndian::write_u16(&mut bytes[..], v);
        bytes
    }

    pub fn string(s: &CqlString, buf: &mut Vec<u8>) {
        buf.extend(&short(s.len())[..]);
        buf.extend(s.as_bytes());
    }

    pub fn string_list(l: &CqlStringList, buf: &mut Vec<u8>) {
        buf.extend(&short(l.len())[..]);
        for s in l.iter() {
            string(s, buf);
        }
    }

    pub fn string_multimap(m: &CqlStringMultiMap, buf: &mut Vec<u8>) {
        buf.extend(&short(m.len())[..]);
        for (k, lst) in m.iter() {
            string(k, buf);
            string_list(lst, buf);
        }
    }
}

#[cfg(test)]
mod test {
    use super::{encode, decode, CqlString, CqlStringList, CqlStringMultiMap};
    use std::collections::HashMap;

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
        let sl = CqlStringList::try_from(sl).unwrap();

        let mut buf = Vec::new();
        encode::string_list(&sl, &mut buf);
        assert_finished_and_eq!(decode::string_list(&buf), sl);
    }

    #[test]
    fn string_multimap() {
        let sla = ["a", "b"];
        let slb = ["c", "d"];
        let mut mm = HashMap::new();
        let sl = CqlStringList::try_from_iter(&sla).unwrap();
        mm.insert(CqlString::try_from("a").unwrap(), sl);

        let sl = CqlStringList::try_from_iter(&slb).unwrap();
        mm.insert(CqlString::try_from("b").unwrap(), sl);

        let smm = CqlStringMultiMap::try_from(mm).unwrap();

        let mut buf = Vec::new();
        encode::string_multimap(&smm, &mut buf);

        assert_finished_and_eq!(decode::string_multimap(&buf), smm);
    }
}
