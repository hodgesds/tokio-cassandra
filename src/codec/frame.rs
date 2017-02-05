use codec::header::Header;

struct Frame<T> {
    pub header: Header,
    pub body: T,
}

enum Operation {

}

mod requests {
    use std::io;
    struct OptionsRequest;
    use codec::header::OpCode;

    trait CqlProtoEncode<W> {
        fn opcode() -> OpCode;
        fn encode(&self, f: &mut W) -> Result<usize, io::Error>;
    }

    impl<W> CqlProtoEncode<W> for OptionsRequest
        where W: io::Write
    {
        fn opcode() -> OpCode {
            OpCode::Options
        }

        fn encode(&self, f: &mut W) -> Result<usize, io::Error> {
            Ok(0)
        }
    }


}
struct StaticBody;
struct StreamBody;

// let f = Frame.from(OptionsRequest)
// f.write()
