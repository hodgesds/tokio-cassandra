use std::io;

error_chain! {
    errors{
        CqlError(code: i32, msg: String) {
            description("Cql error message from server")
            display("CQL Server Error({}): {}", code, msg)
        }
        HandshakeError(msg: String)
    }

    foreign_links{
        IoErr(io::Error);
    }
}
