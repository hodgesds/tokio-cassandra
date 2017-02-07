pub mod decode {
    use nom::{IResult, be_u16};

    pub fn short(b: &[u8]) -> IResult<&[u8], u16> {
        be_u16(b)
    }

    #[cfg(test)]
    mod test {
        use byteorder::{ByteOrder, BigEndian};
        use super::super::decode;

        #[test]
        fn short() {
            let mut buf = [0u8; 2];
            let expected = 342;
            BigEndian::write_u16(&mut buf[..], expected);

            assert_eq!(expected, decode::short(&buf).unwrap().1);
        }
    }
}
