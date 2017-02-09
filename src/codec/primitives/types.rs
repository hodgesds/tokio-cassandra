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
              S: Into<&'a str>
    {
        let v = v.into_iter();
        let mut res = Vec::with_capacity(v.len());
        for s in v {
            res.push(CqlString::try_from(s.into())?);
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
pub struct CqlStringMap<'a> {
    container: HashMap<CqlString<'a>, CqlString<'a>>,
}

impl<'a> CqlStringMap<'a> {
    pub fn try_from(map: HashMap<CqlString<'a>, CqlString<'a>>) -> Result<CqlStringMap<'a>> {
        match map.len() > u16::max_value() as usize {
            true => Err(ErrorKind::MaximumLengthExceeded(map.len()).into()),
            false => Ok(CqlStringMap { container: map }),
        }
    }

    pub fn try_from_iter<I, E>(v: I) -> Result<CqlStringMap<'a>>
        where I: IntoIterator<IntoIter = E, Item = (CqlString<'a>, CqlString<'a>)>,
              E: Iterator<Item = (CqlString<'a>, CqlString<'a>)> + ExactSizeIterator
    {
        let v = v.into_iter();
        let mut res = HashMap::with_capacity(v.len());
        for (k, v) in v {
            res.insert(k, v);
        }
        CqlStringMap::try_from(res)
    }

    pub unsafe fn unchecked_from(lst: HashMap<CqlString<'a>, CqlString<'a>>) -> CqlStringMap<'a> {
        CqlStringMap { container: lst }
    }

    pub fn len(&self) -> u16 {
        self.container.len() as u16
    }

    pub fn iter(&'a self) -> ::std::collections::hash_map::Iter<'a, CqlString<'a>, CqlString<'a>> {
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

    pub fn try_from_iter<I, E>(v: I) -> Result<CqlStringMultiMap<'a>>
        where I: IntoIterator<IntoIter = E, Item = (CqlString<'a>, CqlStringList<'a>)>,
              E: Iterator<Item = (CqlString<'a>, CqlStringList<'a>)> + ExactSizeIterator
    {
        let v = v.into_iter();
        let mut res = HashMap::with_capacity(v.len());
        for (k, v) in v {
            res.insert(k, v);
        }
        CqlStringMultiMap::try_from(res)
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

#[cfg(test)]
mod test {
    use super::{CqlString, CqlStringList, CqlStringMap, CqlStringMultiMap};
    use super::super::{encode, decode};

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
