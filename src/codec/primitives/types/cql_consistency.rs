use super::*;

#[derive(Debug, PartialEq, Eq)]
pub enum Consistency {
    ANY,
    ONE,
    TWO,
    THREE,
    QUORUM,
    ALL,
    LOCAL_QUORUM,
    EACH_QUORUM,
    SERIAL,
    LOCAL_SERIAL,
    LOCAL_ONE,
}

impl Consistency {
    pub fn try_from(short: u16) -> Result<Consistency> {
        Ok(match short {
            0x0000 => Consistency::ANY,
            0x0001 => Consistency::ONE,
            0x0002 => Consistency::TWO,
            0x0003 => Consistency::THREE,
            0x0004 => Consistency::QUORUM,
            0x0005 => Consistency::ALL,
            0x0006 => Consistency::LOCAL_QUORUM,
            0x0007 => Consistency::EACH_QUORUM,
            0x0008 => Consistency::SERIAL,
            0x0009 => Consistency::LOCAL_SERIAL,
            0x000A => Consistency::LOCAL_ONE,
            _ => return Err("Unknown Consistency".into()),
        })
    }

    pub fn as_short(&self) -> u16 {
        use self::Consistency::*;
        match *self {
            ANY => 0x0000,
            ONE => 0x0001,
            TWO => 0x0002,
            THREE => 0x0003,
            QUORUM => 0x0004,
            ALL => 0x0005,
            LOCAL_QUORUM => 0x0006,
            EACH_QUORUM => 0x0007,
            SERIAL => 0x0008,
            LOCAL_SERIAL => 0x0009,
            LOCAL_ONE => 0x000A,
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use super::super::super::{encode, decode};

    #[test]
    fn consistency() {
        let c = Consistency::ANY;
        let buf = encode::consistency(&c);
        let buf = Vec::from(&buf[..]).into();

        let res = decode::consistency(buf);
        assert_eq!(res.unwrap().1, c);
    }

}

//0x0000    ANY
//0x0001    ONE
//0x0002    TWO
//0x0003    THREE
//0x0004    QUORUM
//0x0005    ALL
//0x0006    LOCAL_QUORUM
//0x0007    EACH_QUORUM
//0x0008    SERIAL
//0x0009    LOCAL_SERIAL
//0x000A    LOCAL_ONE
