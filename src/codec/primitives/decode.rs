use super::{CqlStringList, CqlString, CqlStringMap, CqlStringMultiMap};
use std::collections::HashMap;
use tokio_core::io::EasyBuf;
use byteorder::{ByteOrder, BigEndian};
use codec::primitives::CqlFrom;

#[derive(Debug,PartialEq,Eq,Clone,Copy)]
pub enum Needed {
    /// needs more data, but we do not know how much
    Unknown,
    /// contains the total required data size, as opposed to the size still needed
    Size(usize),
}


// Maybe replace by error_chain
quick_error! {
    #[derive(Debug, PartialEq, Eq, Clone)]
    pub enum Error {
        Incomplete(n: Needed) {
            description("Unsufficient bytes")
            display("Buffer contains unsufficient bytes, needed {:?}", n)
        }
    }
}

use self::Error::*;
use self::Needed::*;

pub type ParseResult<T> = Result<(EasyBuf, T), Error>;

pub fn short(mut i: EasyBuf) -> ParseResult<u16> {
    if i.len() < 2 {
        return Err(Incomplete(Size(2)));
    }
    let databuf = i.drain_to(2);
    let short = BigEndian::read_u16(databuf.as_slice());
    Ok((i, short))
}

pub fn string(buf: EasyBuf) -> ParseResult<CqlString<EasyBuf>> {
    let (mut buf, len) = short(buf)?;
    if buf.len() < len as usize {
        return Err(Incomplete(Size(len as usize)));
    }
    let str = CqlString::from(buf.drain_to(len as usize));
    Ok((buf, str))
}

pub fn string_list(i: EasyBuf) -> ParseResult<CqlStringList<EasyBuf>> {
    let (mut buf, len) = short(i)?;
    let mut v = Vec::new();
    for _ in 0..len {
        let (nb, str) = string(buf)?;
        buf = nb;
        v.push(str);
    }
    let lst = unsafe { CqlStringList::unchecked_from(v) };
    Ok((buf, lst))
}

pub fn string_map(i: EasyBuf) -> ParseResult<CqlStringMap<EasyBuf>> {
    let (mut buf, len) = short(i)?;
    let mut map = HashMap::new();

    for _ in 0..len {
        let (nb, key) = string(buf)?;
        buf = nb;
        let (nb, value) = string(buf)?;
        buf = nb;
        map.insert(key, value);
    }

    Ok((buf, unsafe { CqlStringMap::unchecked_from(map) }))
}

pub fn string_multimap(i: EasyBuf) -> ParseResult<CqlStringMultiMap<EasyBuf>> {
    let (mut buf, len) = short(i)?;
    let mut map = HashMap::new();

    for _ in 0..len {
        let (nb, key) = string(buf)?;
        buf = nb;
        let (nb, value) = string_list(buf)?;
        buf = nb;
        map.insert(key, value);
    }

    Ok((buf, unsafe { CqlStringMultiMap::unchecked_from(map) }))
}

mod test {
    use super::*;
    use super::super::encode;
    //    use byteorder::{ByteOrder, BigEndian};
    //    use tokio_core::io::EasyBuf;

    #[test]
    fn short_incomplete() {
        assert_eq!(short(vec![0].into()).unwrap_err(), Incomplete(Size(2)));
    }

    #[test]
    fn short_complete() {
        use std::ops::DerefMut;
        let mut b: EasyBuf = vec![0u8, 1, 2, 3, 4].into();
        let b2 = b.clone();
        let expected = 16723;
        BigEndian::write_u16(&mut b.get_mut().deref_mut(), expected);
        let (nb, res) = short(b).unwrap();
        assert_eq!(res, expected);
        assert_eq!(nb.as_slice(), &b2.as_slice()[2..]);
    }

    #[test]
    fn string_complete() {
        let s = CqlString::try_from("hello").unwrap();
        let ofs = 5usize;
        let mut b: Vec<_> = (0u8..ofs as u8).collect();
        encode::string(&s, &mut b);
        b.extend(0..2);
        let mut e: EasyBuf = b.into();
        e.drain_to(ofs);
        let (e, str) = string(e).unwrap();
        assert_eq!(e.len(), 2);
        assert_eq!(str, s);
    }

    #[test]
    fn string_incomplete() {
        let s: CqlString<EasyBuf> = CqlString::try_from("hello").unwrap();
        let mut b: Vec<_> = Vec::new();
        encode::string(&s, &mut b);
        let e: EasyBuf = b.into();

        assert_eq!(string(e.clone().drain_to(1)).unwrap_err(),
                   Incomplete(Size(2)));
        assert_eq!(string(e.clone().drain_to(3)).unwrap_err(),
                   Incomplete(Size(5)));
    }

    // TODO: move tests from types here, cause it seems very similar

    //
    //    #[test]
    //    fn string_list_incomplete_and_complete() {
    //        let vs = vec!["hello", "world"];
    //        let v = borrowed::CqlStringList::try_from_iter(vs.iter().cloned()).unwrap();
    //        let ofs = 5usize;
    //        let mut b: Vec<_> = (0u8..ofs as u8).collect();
    //        encode::string_list(&v, &mut b);
    //        let sls = b.len() - ofs;
    //        b.extend(1..2);
    //
    //        assert_eq!(string_list(ofs, &b[ofs..ofs + 1]), Err(Incomplete(Size(2))));
    //        assert_eq!(string_list(ofs, &b[ofs..ofs + 2]), Err(Incomplete(Unknown)));
    //        assert_eq!(string_list(ofs, &b[ofs..]),
    //        Ok((&b[ofs + sls..],
    //            indexed::CqlStringList {
    //                at: ofs + 2,
    //                len: 2,
    //            })));
    //    }
}
