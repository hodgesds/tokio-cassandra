use super::super::indexed;
use super::super::borrowed;
use nom::IResult;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CqlString<'a> {
    borrowed: borrowed::CqlString<'a>,
    buf: &'a [u8],
}

impl<'a> CqlString<'a> {
    pub fn unchecked_from(buf: &'a [u8], s: &'a str) -> IResult<&'a [u8], CqlString<'a>> {
        IResult::Done(buf,
                      CqlString {
                          borrowed: unsafe { borrowed::CqlString::unchecked_from(s) },
                          buf: buf,
                      })
    }
}
