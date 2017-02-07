pub mod decode {
    use nom::{IResult, be_u16};

    pub fn short(b: &[u8]) -> IResult<&[u8], u16> {
        be_u16(b)
    }

    named!(string(u8) -> &str, take_str!(take!(u16!())));


    #[cfg(test)]
    mod test {
        use byteorder::{ByteOrder, BigEndian};
        use super::super::decode;

        #[test]
        fn short() {
            let mut buf = [0u8; 2];
            let expected = 342;
            BigEndian::write_u16(&mut buf[..], expected);

            assert_finished_and_eq!(decode::short(&buf), expected);
        }

        #[test]
        fn string() {
            let s = "Hello üß";
            let mut len = [0u8; 2];
            BigEndian::write_u16(&mut len[..], s.len() as u16);

            let buf = Vec::new();
            buf.extend(len);
            buf.extend(s.as_bytes());

            assert_finished_and_eq!(decode::string(&buf), s);
        }
    }
}
