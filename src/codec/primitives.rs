pub mod decode {
    use nom::be_u16;

    named!(pub short(&[u8]) -> u16, call!(be_u16));
    named!(pub string(&[u8]) -> &str, do_parse!(
            s: short          >>
            str: take_str!(s) >>
            (str)
            )
    );


}

pub mod encode {
    use byteorder::{ByteOrder, BigEndian};

    pub fn short(v: u16) -> [u8; 2] {
        let mut bytes = [0u8; 2];
        BigEndian::write_u16(&mut bytes[..], v);
        bytes
    }

    pub fn string(s: &str) -> Vec<u8> {
        let mut len = [0u8; 2];
        BigEndian::write_u16(&mut len[..], s.len() as u16);

        let mut buf: Vec<u8> = Vec::new();
        buf.extend(&len[..]);
        buf.extend(s.as_bytes());
        buf
    }
}

#[cfg(test)]
mod test {
    use super::decode;
    use super::encode;

    #[test]
    fn short() {
        let expected: u16 = 342;
        let buf = encode::short(expected);

        assert_finished_and_eq!(decode::short(&buf), expected);
    }

    #[test]
    fn string() {
        let s = "Hello üß";
        let buf = encode::string(s);

        assert_finished_and_eq!(decode::string(&buf), s);
    }
}
