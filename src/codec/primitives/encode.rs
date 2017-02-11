use byteorder::{ByteOrder, BigEndian};
use super::borrowed::{CqlStringList, CqlString, CqlStringMap, CqlStringMultiMap};

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

pub fn string_map(m: &CqlStringMap, buf: &mut Vec<u8>) {
    buf.extend(&short(m.len())[..]);
    for (k, v) in m.iter() {
        string(k, buf);
        string(v, buf);
    }
}

pub fn string_multimap(m: &CqlStringMultiMap, buf: &mut Vec<u8>) {
    buf.extend(&short(m.len())[..]);
    for (k, lst) in m.iter() {
        string(k, buf);
        string_list(lst, buf);
    }
}
