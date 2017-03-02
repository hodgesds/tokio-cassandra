use codec::primitives::{CqlFrom, CqlString, CqlBytes, CqlStringList, CqlStringMultiMap};
use codec::header::ProtocolVersion;
use codec::primitives::decode;
use tokio_core::io::EasyBuf;
use semver::Version;

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        HeaderError(::codec::header::Error);
        DecodeError(::codec::primitives::decode::Error);
    }

    errors {
        Incomplete(err: String) {
            description("Unsufficient bytes")
            display("Buffer contains unsufficient bytes: {}", err)
        }
        ParserError(err: String) {
            description("Error during parsing")
            display("{}", err)
        }
    }
}

#[derive(Debug)]
pub enum Message {
    Supported(SupportedMessage),
    Ready,
    Authenticate(AuthenticateMessage),
    AuthSuccess(AuthSuccessMessage),
    Error(ErrorMessage),
    Result,
}

pub trait CqlDecode<T> {
    fn decode(v: ProtocolVersion, buf: ::tokio_core::io::EasyBuf) -> Result<T>;
}

#[derive(Debug)]
pub struct SupportedMessage(pub CqlStringMultiMap<EasyBuf>);

impl SupportedMessage {
    pub fn cql_versions(&self) -> Option<&CqlStringList<EasyBuf>> {
        self.0.get(unsafe { &CqlString::unchecked_from("CQL_VERSION") })
    }

    pub fn compression(&self) -> Option<&CqlStringList<EasyBuf>> {
        self.0.get(unsafe { &CqlString::unchecked_from("COMPRESSION") })
    }

    pub fn latest_cql_version(&self) -> Option<&CqlString<EasyBuf>> {
        self.cql_versions()
            .and_then(|lst| {
                lst.iter()
                    .filter_map(|v| Version::parse(v.as_ref()).ok().map(|vp| (vp, v)))
                    .max_by_key(|t| t.0.clone())
                    .map(|(_vp, v)| v)
            })
    }
}

impl CqlDecode<SupportedMessage> for SupportedMessage {
    fn decode(_v: ProtocolVersion, buf: ::tokio_core::io::EasyBuf) -> Result<SupportedMessage> {
        decode::string_multimap(buf)
            .map(|d| d.1.into())
            .map_err(|err| ErrorKind::ParserError(format!("{}", err)).into())
    }
}

impl From<CqlStringMultiMap<::tokio_core::io::EasyBuf>> for SupportedMessage {
    fn from(v: CqlStringMultiMap<::tokio_core::io::EasyBuf>) -> Self {
        SupportedMessage(v)
    }
}

#[derive(Debug)]
pub struct AuthenticateMessage {
    pub authenticator: CqlString<EasyBuf>,
}

impl CqlDecode<AuthenticateMessage> for AuthenticateMessage {
    fn decode(_v: ProtocolVersion, buf: ::tokio_core::io::EasyBuf) -> Result<AuthenticateMessage> {
        decode::string(buf)
            .map(|d| AuthenticateMessage { authenticator: d.1 })
            .map_err(|err| ErrorKind::ParserError(format!("{}", err)).into())
    }
}

#[derive(Debug)]
pub struct AuthSuccessMessage {
    pub payload: CqlBytes<EasyBuf>,
}

impl CqlDecode<AuthSuccessMessage> for AuthSuccessMessage {
    fn decode(_v: ProtocolVersion, buf: ::tokio_core::io::EasyBuf) -> Result<AuthSuccessMessage> {
        decode::bytes(buf)
            .map(|d| AuthSuccessMessage { payload: d.1 })
            .map_err(|err| ErrorKind::ParserError(format!("{}", err)).into())
    }
}

#[derive(Debug)]
pub struct ErrorMessage {
    pub code: i32,
    pub text: CqlString<EasyBuf>,
}

impl CqlDecode<ErrorMessage> for ErrorMessage {
    fn decode(_v: ProtocolVersion, buf: ::tokio_core::io::EasyBuf) -> Result<ErrorMessage> {
        let (buf, code) = decode::int(buf)?;
        let (_, text) = decode::string(buf)?;
        Ok(ErrorMessage {
            code: code,
            text: text,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ResultHeader {
    Void,
    SetKeyspace(CqlString<EasyBuf>),
    SchemaChange(SchemaChangePayload),
    Rows(RowsMetadata),
}

#[derive(Debug, PartialEq, Eq)]
pub struct SchemaChangePayload {
    change_type: CqlString<EasyBuf>,
    target: CqlString<EasyBuf>,
    options: CqlString<EasyBuf>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RowsMetadata {
    global_tables_spec: Option<TableSpec>,
    paging_state: Option<CqlBytes<EasyBuf>>,
    no_metadata: bool,
    columns_count: i32,
}

impl Default for RowsMetadata {
    fn default() -> RowsMetadata {
        RowsMetadata {
            global_tables_spec: None,
            paging_state: None,
            no_metadata: false,
            columns_count: -1,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct TableSpec {
    keyspace: CqlString<EasyBuf>,
    table: CqlString<EasyBuf>,
}

impl ResultHeader {
    pub fn decode(_v: ProtocolVersion,
                  buf: ::tokio_core::io::EasyBuf)
                  -> Result<Option<ResultHeader>> {

        if buf.len() < 4 {
            Ok(None)
        } else {
            let (buf, t) = decode::int(buf)?;
            match t {
                0x0001 => Ok(Some(ResultHeader::Void)),
                0x0002 => {
                    Self::match_decode(Self::decode_rows_metadata(buf), |d| ResultHeader::Rows(d))
                }
                0x0003 => Self::match_decode(decode::string(buf), |s| ResultHeader::SetKeyspace(s)),
                0x0005 => {
                    Self::match_decode(Self::decode_schema_change(buf),
                                       |c| ResultHeader::SchemaChange(c))
                }
                // TODO:
                // 0x0004    Prepared: result to a PREPARE message.
                _ => Ok(None),
            }
        }
    }

    fn match_decode<T, F>(decoded: decode::ParseResult<T>, f: F) -> Result<Option<ResultHeader>>
        where F: Fn(T) -> ResultHeader
    {
        match decoded {
            Ok((_, s)) => Ok(Some(f(s))),
            Err(decode::Error::Incomplete(_)) => Ok(None),
            Err(a) => Err(a.into()),
        }
    }

    fn decode_schema_change(buf: EasyBuf) -> decode::ParseResult<SchemaChangePayload> {
        let (buf, change_type) = decode::string(buf)?;
        let (buf, target) = decode::string(buf)?;
        let (buf, options) = decode::string(buf)?;

        Ok((buf,
            SchemaChangePayload {
                change_type: change_type,
                target: target,
                options: options,
            }))
    }

    fn decode_rows_metadata(buf: EasyBuf) -> decode::ParseResult<RowsMetadata> {
        let (buf, flags) = decode::int(buf)?;
        let (buf, col_count) = decode::int(buf)?;

        // <flags><columns_count>[<paging_state>]
        // [<global_table_spec>?<col_spec_1>...<col_spec_n>]

        let mut rows_metadata = RowsMetadata::default();

        rows_metadata.columns_count = col_count;

        if (flags & 0x0002) == 0x0002 {
            rows_metadata.paging_state = Some(cql_bytes!(1, 2, 3));
        }

        let buf = if (flags & 0x0001) == 0x0001 {
            let (buf, keyspace) = decode::string(buf)?;
            let (buf, table) = decode::string(buf)?;
            rows_metadata.global_tables_spec = Some(TableSpec {
                keyspace: keyspace,
                table: table,
            });
            buf
        } else {
            buf
        };

        // TODO: parse column spec if present

        rows_metadata.no_metadata = (flags & 0x0004) == 0x0004;

        Ok((buf, rows_metadata))
    }
}

#[cfg(test)]
mod test {
    use codec::header::Header;
    use codec::header::ProtocolVersion::*;
    use codec::primitives::{CqlStringMultiMap, CqlStringList, CqlString};
    use super::*;

    fn skip_header(b: &[u8]) -> &[u8] {
        &b[Header::encoded_len()..]
    }

    #[test]
    fn decode_supported_message() {
        let msg = include_bytes!("../../tests/fixtures/v3/responses/supported.msg");
        let buf = Vec::from(skip_header(&msg[..])).into();
        let res = SupportedMessage::decode(Version3, buf).unwrap();

        let sla = ["3.2.1"];
        let slb = ["snappy", "lz4"];
        let csl1 = CqlStringList::try_from_iter_easy(sla.iter().cloned()).unwrap();
        let csl2 = CqlStringList::try_from_iter_easy(slb.iter().cloned()).unwrap();

        assert_eq!(res.cql_versions().unwrap(), &csl1);
        assert_eq!(res.compression().unwrap(), &csl2);
    }

    #[test]
    fn supported_message_latest_cql_version() {
        let versions = ["3.2.1", "3.1.2", "4.0.1"];
        let vm = CqlStringList::try_from_iter_easy(versions.iter().cloned()).unwrap();
        let smm = CqlStringMultiMap::try_from_iter(vec![(CqlString::try_from("CQL_VERSION")
                                                             .unwrap(),
                                                         vm)])
            .unwrap();
        let msg = SupportedMessage::from(smm);

        assert_eq!(msg.latest_cql_version(),
                   Some(&CqlString::try_from("4.0.1").unwrap()));
    }

    #[test]
    fn decode_authenticate_message() {
        let msg = include_bytes!("../../tests/fixtures/v3/responses/authenticate.msg");
        let buf = Vec::from(skip_header(&msg[..])).into();
        let res = AuthenticateMessage::decode(Version3, buf).unwrap();

        let authenticator = CqlString::try_from("org.apache.cassandra.auth.PasswordAuthenticator")
            .unwrap();

        assert_eq!(res.authenticator, authenticator);
    }

    #[test]
    fn decode_auth_success_message() {
        let msg = include_bytes!("../../tests/fixtures/v3/responses/auth_success.msg");
        let buf = Vec::from(skip_header(&msg[..])).into();
        let res = AuthSuccessMessage::decode(Version3, buf).unwrap();

        assert_eq!(res.payload.as_bytes(), None);
    }

    #[test]
    fn decode_error_message() {
        let msg = include_bytes!("../../tests/fixtures/v3/responses/error_credentials.msg");
        let buf = Vec::from(skip_header(&msg[..])).into();
        let res = ErrorMessage::decode(Version3, buf).unwrap();

        assert_eq!(res.code, 256);
        assert_eq!(res.text,
                   CqlString::try_from("Username and/or password are incorrect").unwrap());
    }

    #[test]
    fn decode_result_header_rows() {
        let msg = include_bytes!("../../tests/fixtures/v3/responses/result_rows.msg");
        let buf = Vec::from(skip_header(&msg[..]));

        // Ok(None) Ok(Some()), Err()
        let res = ResultHeader::decode(Version3, Vec::from(&buf[0..5]).into()).unwrap();
        assert_eq!(res, None);

        let rexpected = RowsMetadata {
            global_tables_spec: Some(TableSpec {
                keyspace: cql_string!("system"),
                table: cql_string!("local"),
            }),
            paging_state: None,
            no_metadata: false,
            columns_count: 18,
        };

        let res = ResultHeader::decode(Version3, buf.into()).unwrap();
        assert_eq!(res, Some(ResultHeader::Rows(rexpected)));

        // rest of drained buf should be used for streaming results after that
    }

    #[test]
    fn decode_result_header_void() {
        let msg = include_bytes!("../../tests/fixtures/v3/responses/result_void.msg");
        let buf = Vec::from(skip_header(&msg[..]));

        // Ok(None) Ok(Some()), Err()
        let res = ResultHeader::decode(Version3, Vec::from(&buf[0..1]).into()).unwrap();
        assert_eq!(res, None);

        let res = ResultHeader::decode(Version3, buf.into()).unwrap();
        assert_eq!(res, Some(ResultHeader::Void));
    }

    #[test]
    fn decode_result_header_set_keyspace() {
        let msg = include_bytes!("../../tests/fixtures/v3/responses/result_set_keyspace.msg");
        let buf = Vec::from(skip_header(&msg[..]));

        // Ok(None) Ok(Some()), Err()
        let res = ResultHeader::decode(Version3, Vec::from(&buf[0..6]).into()).unwrap();
        assert_eq!(res, None);

        let res = ResultHeader::decode(Version3, Vec::from(&buf[0..9]).into()).unwrap();
        assert_eq!(res, None);

        let res = ResultHeader::decode(Version3, buf.into()).unwrap();
        assert_eq!(res, Some(ResultHeader::SetKeyspace(cql_string!("abcd"))));
    }

    #[test]
    fn decode_result_header_schema_change() {
        let msg = include_bytes!("../../tests/fixtures/v3/responses/result_schema_change.msg");
        let buf = Vec::from(skip_header(&msg[..]));

        // Ok(None) Ok(Some()), Err()
        let res = ResultHeader::decode(Version3, Vec::from(&buf[0..6]).into()).unwrap();
        assert_eq!(res, None);

        let res = ResultHeader::decode(Version3, buf.into()).unwrap();
        assert_eq!(res,
                   Some(ResultHeader::SchemaChange(SchemaChangePayload {
                       change_type: cql_string!("change_type"),
                       target: cql_string!("target"),
                       options: cql_string!("options"),
                   })));
    }
}
