use super::indexed;


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

pub type ParseResult<'a, T> = Result<(&'a [u8], T), Error>;

use self::Error::*;
use self::Needed::*;


pub fn short(i: &[u8]) -> ParseResult<u16> {
    if i.len() < 2 {
        return Err(Incomplete(Size(2)));
    }
    let res = ((i[0] as u16) << 8) + i[1] as u16;
    Ok((&i[2..], res))
}

pub fn string(ofs: usize, i: &[u8]) -> ParseResult<indexed::CqlString> {
    let (i, s) = short(i)?;
    if i.len() < s as usize {
        return Err(Incomplete(Size(s as usize)));
    }
    Ok((&i[s as usize..],
        indexed::CqlString {
            at: ofs + 2,
            len: s,
        }))
}


#[cfg(test)]
mod test {
    use super::*;
    use super::super::{indexed, encode, borrowed};
    use byteorder::{ByteOrder, BigEndian};

    # [test]
    fn short_incomplete() {
        assert_eq!(short(&[0]), Err(Incomplete(Size(2))));
    }

    #[test]
    fn short_complete() {
        let mut b = [0u8, 1, 2, 3, 4];
        let expected = 16723;
        BigEndian::write_u16(&mut b, expected);
        assert_eq!(short(&b[..]), Ok((&b[2..], expected)));
    }

    #[test]
    fn string_complete() {
        let s = borrowed::CqlString::try_from("hello").unwrap();
        let ofs = 5;
        let mut b: Vec<_> = (0u8..5).collect();
        encode::string(&s, &mut b);
        b.extend(0..2);

        assert_eq!(string(ofs, &b[ofs..ofs + 1]), Err(Incomplete(Size(2))));
        assert_eq!(string(ofs, &b[ofs..ofs + 4]), Err(Incomplete(Size(5))));
        assert_eq!(string(ofs, &b[ofs..]),
                   Ok((&b[ofs + 2 + 5..],
                       indexed::CqlString {
                           at: ofs + 2,
                           len: 5,
                       })));
    }
}
