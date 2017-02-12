use std::collections::HashMap;
use std::hash::{Hasher, Hash};
use std::convert::AsRef;
use std::fmt::{Formatter, Debug};
use tokio_core::io::EasyBuf;

error_chain! {
    errors {
        MaximumLengthExceeded(l: usize) {
          description("Too many elements container")
          display("Expected not more than {} elements, got {}.", u16::max_value(), l)
        }
    }
}

#[derive(Clone)]
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

impl<T> PartialEq for CqlString<T>
    where T: AsRef<[u8]>
{
    fn eq(&self, other: &CqlString<T>) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl<T> Eq for CqlString<T> where T: AsRef<[u8]> {}

impl<T> AsRef<str> for CqlString<T>
    where T: AsRef<[u8]>
{
    fn as_ref(&self) -> &str {
        ::std::str::from_utf8(&self.buf.as_ref()).unwrap()
    }
}

impl<T> Hash for CqlString<T>
    where T: AsRef<[u8]>
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}

pub trait HasLength {
    fn length(&self) -> usize;
}

impl<'a> HasLength for &'a str {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<T> HasLength for Vec<T> {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<T, U> HasLength for HashMap<T, U>
    where T: ::std::cmp::Eq + Hash
{
    fn length(&self) -> usize {
        self.len()
    }
}

impl CqlString<::tokio_core::io::EasyBuf> {
    pub fn from(buf: ::tokio_core::io::EasyBuf) -> CqlString<::tokio_core::io::EasyBuf> {
        CqlString { buf: buf }
    }
}


pub trait CqlFrom<C, V>
    where V: HasLength
{
    fn try_from(s: V) -> Result<C> {
        match s.length() > u16::max_value() as usize {
            true => Err(ErrorKind::MaximumLengthExceeded(s.length()).into()),
            false => {
                Ok({
                    unsafe { Self::unchecked_from(s) }
                })
            }
        }
    }
    unsafe fn unchecked_from(s: V) -> C;
}

impl<'a> CqlFrom<CqlString<EasyBuf>, &'a str> for CqlString<EasyBuf> {
    unsafe fn unchecked_from(s: &str) -> CqlString<EasyBuf> {
        let vec = Vec::from(s);
        CqlString { buf: vec.into() }
    }
}

impl<'a> CqlFrom<CqlString<Vec<u8>>, &'a str> for CqlString<Vec<u8>> {
    unsafe fn unchecked_from(s: &str) -> CqlString<Vec<u8>> {
        let vec = Vec::from(s);
        CqlString { buf: vec }
    }
}

impl<T> CqlString<T>
    where T: AsRef<[u8]>
{
    pub fn len(&self) -> u16 {
        self.buf.as_ref().len() as u16 // TODO: safe cast
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.buf.as_ref()
    }
}

/// TODO: zero copy - implement it as offset to beginning of vec to CqlStrings to prevent Vec
/// allocation
#[derive(Debug, Eq, Clone)]
pub struct CqlStringList<T>
    where T: AsRef<[u8]>
{
    container: Vec<CqlString<T>>,
}

impl<T> PartialEq for CqlStringList<T>
    where T: AsRef<[u8]>
{
    fn eq(&self, other: &CqlStringList<T>) -> bool {
        self.container == other.container
    }
}

impl<T> CqlFrom<CqlStringList<T>, Vec<CqlString<T>>> for CqlStringList<T>
    where T: AsRef<[u8]>
{
    unsafe fn unchecked_from(lst: Vec<CqlString<T>>) -> CqlStringList<T> {
        CqlStringList { container: lst }
    }
}

impl CqlStringList<EasyBuf> {
    pub fn try_from_iter_easy<'a, I, E, S>(v: I) -> Result<CqlStringList<EasyBuf>>
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
}

impl CqlStringList<Vec<u8>> {
    pub fn try_from_iter<'a, I, E, S>(v: I) -> Result<CqlStringList<Vec<u8>>>
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
}

impl<T> CqlStringList<T>
    where T: AsRef<[u8]>
{
    pub fn len(&self) -> u16 {
        self.container.len() as u16
    }

    pub fn iter(&self) -> ::std::slice::Iter<CqlString<T>> {
        self.container.iter()
    }
}

#[derive(Debug, Eq, Clone)]
pub struct CqlStringMap<T>
    where T: AsRef<[u8]>
{
    container: HashMap<CqlString<T>, CqlString<T>>,
}

impl<T> PartialEq for CqlStringMap<T>
    where T: AsRef<[u8]>
{
    fn eq(&self, other: &CqlStringMap<T>) -> bool {
        self.container == other.container
    }
}

impl<T> CqlFrom<CqlStringMap<T>, HashMap<CqlString<T>, CqlString<T>>> for CqlStringMap<T>
    where T: AsRef<[u8]>
{
    unsafe fn unchecked_from(map: HashMap<CqlString<T>, CqlString<T>>) -> CqlStringMap<T> {
        CqlStringMap { container: map }
    }
}

impl<T> CqlStringMap<T>
    where T: AsRef<[u8]>
{
    pub fn try_from_iter<I, E>(v: I) -> Result<CqlStringMap<T>>
        where I: IntoIterator<IntoIter = E, Item = (CqlString<T>, CqlString<T>)>,
              E: Iterator<Item = (CqlString<T>, CqlString<T>)> + ExactSizeIterator
    {
        let v = v.into_iter();
        let mut res = HashMap::with_capacity(v.len());
        for (k, v) in v {
            res.insert(k, v);
        }
        CqlStringMap::try_from(res)
    }

    pub fn len(&self) -> u16 {
        self.container.len() as u16
    }

    pub fn iter(&self) -> ::std::collections::hash_map::Iter<CqlString<T>, CqlString<T>> {
        self.container.iter()
    }
}

#[derive(Debug, Eq, Clone)]
pub struct CqlStringMultiMap<T>
    where T: AsRef<[u8]>
{
    container: HashMap<CqlString<T>, CqlStringList<T>>,
}

impl<T> PartialEq for CqlStringMultiMap<T>
    where T: AsRef<[u8]>
{
    fn eq(&self, other: &CqlStringMultiMap<T>) -> bool {
        self.container == other.container
    }
}

impl<T> CqlFrom<CqlStringMultiMap<T>, HashMap<CqlString<T>, CqlStringList<T>>>
    for CqlStringMultiMap<T>
    where T: AsRef<[u8]>
{
    unsafe fn unchecked_from(map: HashMap<CqlString<T>, CqlStringList<T>>) -> CqlStringMultiMap<T> {
        CqlStringMultiMap { container: map }
    }
}

impl<T> CqlStringMultiMap<T>
    where T: AsRef<[u8]>
{
    pub fn try_from_iter<I, E>(v: I) -> Result<CqlStringMultiMap<T>>
        where I: IntoIterator<IntoIter = E, Item = (CqlString<T>, CqlStringList<T>)>,
              E: Iterator<Item = (CqlString<T>, CqlStringList<T>)> + ExactSizeIterator
    {
        let v = v.into_iter();
        let mut res = HashMap::with_capacity(v.len());
        for (k, v) in v {
            res.insert(k, v);
        }
        CqlStringMultiMap::try_from(res)
    }

    pub fn len(&self) -> u16 {
        self.container.len() as u16
    }

    pub fn iter(&self) -> ::std::collections::hash_map::Iter<CqlString<T>, CqlStringList<T>> {
        self.container.iter()
    }
}

#[cfg(test)]
mod test {
    use super::{CqlFrom, CqlString, CqlStringList, CqlStringMap, CqlStringMultiMap};
    use super::super::{encode, decode};

    #[test]
    fn short() {
        let expected: u16 = 342;
        let buf = encode::short(expected);
        let buf = Vec::from(&buf[..]).into();

        let res = decode::short(buf);
        assert_eq!(res.unwrap().1, expected);
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
        let buf = Vec::from(&buf[..]).into();

        println!("buf = {:?}", buf);
        let res = decode::string_list(buf).unwrap().1;
        //        assert_eq!(format!("{:?}", res), format!("{:?}", sl));
        assert_eq!(res, sl);
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
        let buf = Vec::from(&buf[..]).into();

        let res = decode::string_map(buf).unwrap().1;
        //        assert_eq!(format!("{:?}", res), format!("{:?}", sm));
        assert_eq!(res, sm);
    }

    #[test]
    fn string_multimap() {
        let sla = ["a", "b"];
        let slb = ["c", "d"];
        let csl1 = CqlStringList::try_from_iter_easy(sla.iter().cloned()).unwrap();
        let csl2 = CqlStringList::try_from_iter_easy(slb.iter().cloned()).unwrap();
        let smm = CqlStringMultiMap::try_from_iter(vec![(CqlString::try_from("a").unwrap(), csl1),
                                                        (CqlString::try_from("b").unwrap(), csl2)])
            .unwrap();

        let mut buf = Vec::new();
        encode::string_multimap(&smm, &mut buf);
        let buf = Vec::from(&buf[..]).into();

        let res = decode::string_multimap(buf).unwrap().1;
        // TODO: TEST without order!!! Maybe Test utils for Rust
        //        assert_eq!(format!("{:?}", res), format!("{:?}", smm));
        assert_eq!(res, smm);
    }
}
