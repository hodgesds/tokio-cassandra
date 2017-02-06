error_chain! {
    foreign_links {
        Io(::std::io::Error);
        HeaderError(::codec::header::Error);
    }
}

enum Body {
    Supported(SupportedBody)
}

struct Response {
    header: Header,
    body: Body
}

pub trait CqlDecode {
    fn decode(&self, f: &[u8]) -> Result<Response>;
}

#[cfg(test)]
mod tests {
    fn decode_supported() {
        let frame = include_bytes!("../../tests/fixtures/v3/srv_supported.msg");
        let r = Response::decode(&frame);
    }
}