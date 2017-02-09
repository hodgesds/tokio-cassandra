use super::{CqlStringList, CqlString, CqlStringMap, CqlStringMultiMap};
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
named!(pub string_map(&[u8]) -> CqlStringMap,
    do_parse!(
        l: short >>
        mm: fold_many_m_n!(l as usize, l as usize,
            do_parse!(
                key: string >>
                value: string >>
                (key, value)
            ),
            HashMap::new(),
            | mut map: HashMap<_,_>, (k, v) | {
                map.insert(k, v);
                map
            }
        )
         >>
        (unsafe { CqlStringMap::unchecked_from(mm) })
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
