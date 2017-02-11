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


#[derive(Debug, Clone)]
pub struct CqlString {
    start: usize,
    end: usize,
    buf: ::tokio_core::io::EasyBuf,
}

impl PartialEq for CqlString {
    fn eq(&self, other: &CqlString) -> bool {
        // TODO: check for reference of EasyBuf too
        self.start == other.start && self.end == other.end
    }
}

impl Eq for CqlString {}

// str::from_utf8(&buf[self.at..
//self.at + self.len as usize]

impl AsRef<str> for CqlString {
    fn as_ref(&self) -> &str {
        ::std::str::from_utf8(&self.buf.as_slice()[self.start..self.end]).unwrap()
    }
}


impl Hash for CqlString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}


impl CqlString {
    pub fn try_from(s: &str) -> Result<CqlString> {
        match s.len() > u16::max_value() as usize {
            true => Err(ErrorKind::MaximumLengthExceeded(s.len()).into()),
            false => {
                Ok({
                    unsafe { CqlString::unchecked_from(s) }
                })
            }
        }
    }

    pub unsafe fn unchecked_from(s: &str) -> CqlString {
        let vec = Vec::from(s);
        let len = vec.len();

        CqlString {
            buf: vec.into(),
            start: 0,
            end: len,
        }
    }

    pub fn len(&self) -> u16 {
        (self.end - self.start) as u16 // TODO: safe cast
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.buf.as_slice()
    }
}

/// TODO: zero copy - implement it as offset to beginning of vec to CqlStrings to prevent Vec
/// allocation
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CqlStringList {
    container: Vec<CqlString>,
}

impl CqlStringList {
    pub fn try_from(lst: Vec<CqlString>) -> Result<CqlStringList> {
        match lst.len() > u16::max_value() as usize {
            true => Err(ErrorKind::MaximumLengthExceeded(lst.len()).into()),
            false => Ok(CqlStringList { container: lst }),
        }
    }

    pub fn try_from_iter<'a, I, E, S>(v: I) -> Result<CqlStringList>
        where I: IntoIterator<IntoIter = E, Item = S>,
              E: Iterator<Item = S> + ExactSizeIterator,
              S: Into<&'a str>
    {
        let v = v.into_iter();
        let mut res = Vec::with_capacity(v.len());
        for s in v {
            res.push(CqlString::try_from(s.into())?);
        }
        CqlStringList::try_from(res)
    }

    pub unsafe fn unchecked_from(lst: Vec<CqlString>) -> CqlStringList {
        CqlStringList { container: lst }
    }

    pub fn len(&self) -> u16 {
        self.container.len() as u16
    }

    pub fn iter(&self) -> ::std::slice::Iter<CqlString> {
        self.container.iter()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CqlStringMap {
    container: HashMap<CqlString, CqlString>,
}

impl CqlStringMap {
    pub fn try_from(map: HashMap<CqlString, CqlString>) -> Result<CqlStringMap> {
        match map.len() > u16::max_value() as usize {
            true => Err(ErrorKind::MaximumLengthExceeded(map.len()).into()),
            false => Ok(CqlStringMap { container: map }),
        }
    }

    pub fn try_from_iter<I, E>(v: I) -> Result<CqlStringMap>
        where I: IntoIterator<IntoIter = E, Item = (CqlString, CqlString)>,
              E: Iterator<Item = (CqlString, CqlString)> + ExactSizeIterator
    {
        let v = v.into_iter();
        let mut res = HashMap::with_capacity(v.len());
        for (k, v) in v {
            res.insert(k, v);
        }
        CqlStringMap::try_from(res)
    }

    pub unsafe fn unchecked_from(lst: HashMap<CqlString, CqlString>) -> CqlStringMap {
        CqlStringMap { container: lst }
    }

    pub fn len(&self) -> u16 {
        self.container.len() as u16
    }

    pub fn iter(&self) -> ::std::collections::hash_map::Iter<CqlString, CqlString> {
        self.container.iter()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CqlStringMultiMap {
    container: HashMap<CqlString, CqlStringList>,
}

impl<'a> CqlStringMultiMap {
    pub fn try_from(map: HashMap<CqlString, CqlStringList>) -> Result<CqlStringMultiMap> {
        match map.len() > u16::max_value() as usize {
            true => Err(ErrorKind::MaximumLengthExceeded(map.len()).into()),
            false => Ok(CqlStringMultiMap { container: map }),
        }
    }

    pub fn try_from_iter<I, E>(v: I) -> Result<CqlStringMultiMap>
        where I: IntoIterator<IntoIter = E, Item = (CqlString, CqlStringList)>,
              E: Iterator<Item = (CqlString, CqlStringList)> + ExactSizeIterator
    {
        let v = v.into_iter();
        let mut res = HashMap::with_capacity(v.len());
        for (k, v) in v {
            res.insert(k, v);
        }
        CqlStringMultiMap::try_from(res)
    }

    pub unsafe fn unchecked_from(lst: HashMap<CqlString, CqlStringList>) -> CqlStringMultiMap {
        CqlStringMultiMap { container: lst }
    }

    pub fn len(&self) -> u16 {
        self.container.len() as u16
    }

    pub fn iter(&'a self) -> ::std::collections::hash_map::Iter<CqlString, CqlStringList> {
        self.container.iter()
    }
}

#[cfg(test)]
mod test {
    use super::{CqlString, CqlStringList, CqlStringMap, CqlStringMultiMap};
    use super::super::super::{encode, decode};

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

        println!("buf = {:?}", buf);

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
    fn string_map() {
        let sm = CqlStringMap::try_from_iter(vec![(CqlString::try_from("a").unwrap(),
                                                   CqlString::try_from("av").unwrap()),
                                                  (CqlString::try_from("a").unwrap(),
                                                   CqlString::try_from("av").unwrap())])
            .unwrap();

        let mut buf = Vec::new();
        encode::string_map(&sm, &mut buf);
        assert_finished_and_eq!(decode::string_map(&buf), sm);
    }

    #[test]
    fn string_multimap() {
        let sla = ["a", "b"];
        let slb = ["c", "d"];
        let csl1 = CqlStringList::try_from_iter(sla.iter().cloned()).unwrap();
        let csl2 = CqlStringList::try_from_iter(slb.iter().cloned()).unwrap();
        let smm = CqlStringMultiMap::try_from_iter(vec![(CqlString::try_from("a").unwrap(), csl1),
                                                        (CqlString::try_from("b").unwrap(), csl2)])
            .unwrap();

        let mut buf = Vec::new();
        encode::string_multimap(&smm, &mut buf);

        assert_finished_and_eq!(decode::string_multimap(&buf), smm);
    }
}
