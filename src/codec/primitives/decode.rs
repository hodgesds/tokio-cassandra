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

pub type ParseResult<'a, T> = Result<(&'a mut EasyBuf, T), Error>;

pub fn short(i: &mut EasyBuf) -> ParseResult<u16> {
    if i.len() < 2 {
        return Err(Incomplete(Size(2)));
    }
    let databuf = i.drain_to(2);
    let short = BigEndian::read_u16(databuf.as_slice());
    Ok((i, short))
}

pub fn string(buf: &mut EasyBuf) -> ParseResult<CqlString<EasyBuf>> {
    let (buf, len) = short(buf)?;
    let str = CqlString::from(buf.drain_to(len as usize));
    Ok((buf, str))
}

pub fn string_list(buf: &mut EasyBuf) -> ParseResult<CqlStringList<EasyBuf>> {
    let (buf, len) = short(buf)?;
    let mut v = Vec::new();
    for _ in 0..len {
        let (_, str) = string(buf)?;
        v.push(str);
    }
    let lst = unsafe { CqlStringList::unchecked_from(v) };
    Ok((buf, lst))
}

pub fn string_map(buf: &mut EasyBuf) -> ParseResult<CqlStringMap<EasyBuf>> {
    let (buf, len) = short(buf)?;
    let mut map = HashMap::new();

    for _ in 0..len {
        let (_, key) = string(buf)?;
        let (_, value) = string(buf)?;
        map.insert(key, value);
    }

    Ok((buf, unsafe { CqlStringMap::unchecked_from(map) }))
}

pub fn string_multimap(buf: &mut EasyBuf) -> ParseResult<CqlStringMultiMap<EasyBuf>> {
    let (buf, len) = short(buf)?;
    let mut map = HashMap::new();

    for _ in 0..len {
        let (_, key) = string(buf)?;
        let (_, value) = string_list(buf)?;
        map.insert(key, value);
    }

    Ok((buf, unsafe { CqlStringMultiMap::unchecked_from(map) }))
}

mod test {
    use super::*;
    //    use super::super::{indexed, encode, borrowed};
    use byteorder::{ByteOrder, BigEndian};
    use tokio_core::io::EasyBuf;

    # [test]
    fn short_incomplete() {
        assert_eq!(short(&mut vec![0].into()).unwrap_err(), Incomplete(Size(2)));
    }

    #[test]
    fn short_complete() {
        use std::ops::DerefMut;
        let mut b: EasyBuf = vec![0u8, 1, 2, 3, 4].into();
        let b2 = b.clone();
        let expected = 16723;
        BigEndian::write_u16(&mut b.get_mut().deref_mut(), expected);
        let (nb, res) = short(&mut b).unwrap();
        assert_eq!(res, expected);
        assert_eq!(nb.as_slice(), &b2.as_slice()[2..]);
    }

    //    #[test]
    //    fn string_incomplete_and_complete() {
    //        let s = borrowed::CqlString::try_from("hello").unwrap();
    //        let ofs = 5usize;
    //        let mut b: Vec<_> = (0u8..ofs as u8).collect();
    //        encode::string(&s, &mut b);
    //        b.extend(0..2);
    //
    //        assert_eq!(string(ofs, &b[ofs..ofs + 1]), Err(Incomplete(Size(2))));
    //        assert_eq!(string(ofs, &b[ofs..ofs + 4]), Err(Incomplete(Size(5))));
    //        assert_eq!(string(ofs, &b[ofs..]),
    //        Ok((&b[ofs + 2 + 5..],
    //            indexed::CqlString {
    //                at: ofs + 2,
    //                len: 5,
    //            })));
    //    }
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
