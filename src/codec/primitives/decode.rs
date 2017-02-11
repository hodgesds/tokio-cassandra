use super::{CqlStringList, CqlString, CqlStringMap, CqlStringMultiMap};
use std::collections::HashMap;
use tokio_core::io::EasyBuf;
use byteorder::{ByteOrder, BigEndian};
use codec::primitives::CqlFrom;

error_chain!{
    errors {
        Incomplete(err: String) {
            description("Unsufficient bytes")
            display("Buffer contains unsufficient bytes: {}", err)
        }
    }
}

pub type ParseResult<'a, T> = Result<(&'a mut EasyBuf, T)>;

pub fn short(buf: &mut EasyBuf) -> ParseResult<u16> {
    let databuf = buf.drain_to(2);
    let short = BigEndian::read_u16(databuf.as_slice());
    Ok((buf, short))
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
