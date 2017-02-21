All stores prefixed with 'client-' can be used for testing
'require_client_auth=true' and force the client to show a valid 
cert when connecting via TLS (I think).

All certificates and stores were created following the instructions here:
http://docs.datastax.com/en/archived/cassandra/2.0/cassandra/security/secureSSLCertificates_t.html
