use super::nom::CqlString;
use nom::be_u16;

named!(pub short(&[u8]) -> u16, call!(be_u16));
named!(pub string(&[u8]) -> CqlString, do_parse!(
        s: short          >>
        str: take_str!(s) >>
        res: call!(CqlString::unchecked_from, str) >>
        (res)
    )
);
