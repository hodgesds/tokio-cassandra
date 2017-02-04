struct CqlCodecV3;



impl Codec for CqlCodecV3 {
    type In = (RequestId, String);
    type Out = (RequestId, String);

    fn decode(&mut self, buf: &mut EasyBuf)
             -> Result<Option<(RequestId, String)>, io::Error>
    {
        // // At least 5 bytes are required for a frame: 4 byte
        // // head + one byte '\n'
        // if buf.len() < 5 {
        //     // We don't yet have a full message
        //     return Ok(None);
        // }

        // // Check to see if the frame contains a new line, skipping
        // // the first 4 bytes which is the request ID
        // let newline = buf.as_ref()[4..].iter().position(|b| *b == b'\n');
        // if let Some(n) = newline {
        //     // remove the serialized frame from the buffer.
        //     let line = buf.drain_to(n + 4);

        //     // Also remove the '\n'
        //     buf.drain_to(1);

        //     // Deserialize the request ID
        //     let id = BigEndian::read_u32(&line.as_ref()[0..4]);

        //     // Turn this data into a UTF string and return it in a Frame.
        //     return match str::from_utf8(&line.as_ref()[4..]) {
        //         Ok(s) => Ok(Some((id as RequestId, s.to_string()))),
        //         Err(_) => Err(io::Error::new(io::ErrorKind::Other,
        //                                      "invalid string")),
        //     }
        // }

        // No `\n` found, so we don't have a complete message
        Ok(None)
    }

    fn encode(&mut self, msg: (RequestId, String),
              buf: &mut Vec<u8>) -> io::Result<()>
    {
        // let (id, msg) = msg;

        // let mut encoded_id = [0; 4];
        // BigEndian::write_u32(&mut encoded_id, id as u32);

        // buf.extend(&encoded_id);
        // buf.extend(msg.as_bytes());
        // buf.push(b'\n');

        Ok(())
    }
}