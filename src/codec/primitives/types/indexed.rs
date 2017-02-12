use super::borrowed;
use std::str;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CqlString {
    pub at: usize,
    pub len: u16,
}



impl CqlString {
    pub fn borrowed<'a>(&self, buf: &'a [u8]) -> borrowed::CqlString<'a> {
        unsafe {
            borrowed::CqlString::unchecked_from(str::from_utf8(&buf[self.at..
                                                                self.at + self.len as usize])
                .unwrap())
        }
    }
}

/// TODO: zero copy - implement it as offset to beginning of vec to CqlStrings to prevent Vec
/// allocation
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CqlStringList {
    pub at: usize,
    pub len: u16,
}

//impl CqlStringList {
//    pub fn borrowed<'a>(&self, buf: &'a [u8]) -> borrowed::CqlStringList<'a> {
//        unsafe {
//            borrowed::CqlStringList::unchecked_from(self.container
//                .iter()
//                .map(|s| s.borrowed(&buf[2..]))
//                .collect())
//        }
//    }
//}

#[cfg(test)]
mod test {
    //    use std::iter::FromIterator;
    //    use super::{CqlStringList, CqlString};
    use super::CqlString;
    use super::super::borrowed;
    //    use super::super::super::encode;

    #[test]
    fn as_cqlstring() {
        let ofs = 1;
        let s = "hello";
        let b = s.as_bytes();
        let cs = CqlString {
            at: ofs,
            len: (b.len() - ofs) as u16,
        };
        assert_eq!(cs.borrowed(b),
                   borrowed::CqlString::try_from(&s[1..]).unwrap())
    }

    //    #[test]
    //    fn as_cqlstringlist_iterator() {
    //        let vs = vec!["hello", "world"];
    //        let v = borrowed::CqlStringList::try_from_iter(vs.iter().cloned()).unwrap();
    //        let ofs = 5;
    //        let mut buf = Vec::from_iter(0..ofs);
    //        encode::string_list(&v, &mut buf);
    //        let iv = CqlStringList {
    //            container: vec![CqlString {
    //                                at: ofs as usize + 2,
    //                                len: vs[0].len() as u16,
    //                            },
    //                            CqlString {
    //                                at: ofs as usize + 2 * 2 + vs[0].len(),
    //                                len: vs[1].len() as u16,
    //                            }],
    //        };
    //        assert_eq!(iv.borrowed(&buf), v);
    //    }
}
