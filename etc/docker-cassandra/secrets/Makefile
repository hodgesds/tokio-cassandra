info:
	$(info A makefile to build all secrets from scratch with default username/passwords)
	$(info Run 'make secrets' to make it happen)
	$(info Run 'make clean' if something has changed and you want to build)

PASSWORD=cassandra
TRUST_STORE=truststore.jks
SERVER_STORE=keystore.jks
SERVER_KEY_FILE=keystore.key.pem
SERVER_CER_FILE=keystore.cer.pem
SERVER_ALIAS=docker-db
SERVER_CERT=$(SERVER_ALIAS).crt
SERVER_PK12=keystore.p12
HOSTNAME=localhost

$(SERVER_STORE):
	keytool -genkeypair -keyalg RSA -alias $(SERVER_ALIAS) -keystore $@ -storepass $(PASSWORD) -keypass $(PASSWORD) -validity 36500 -keysize 2048 -dname "CN=$(HOSTNAME), OU=cluster, O=company, C=DE" 

$(SERVER_CERT): $(SERVER_STORE)
	keytool -export -alias $(SERVER_ALIAS) -keypass $(PASSWORD) -storepass $(PASSWORD) -file $@ -keystore $<

$(TRUST_STORE): $(SERVER_CERT)
	keytool -import -storepass $(PASSWORD) -keypass $(PASSWORD) -v -trustcacerts -alias $(SERVER_ALIAS) -file $< -keystore $@ -noprompt

$(SERVER_PK12): $(SERVER_STORE)
	keytool -importkeystore -srckeystore $< -destkeystore $@ -deststoretype PKCS12 -srcstorepass $(PASSWORD) -deststorepass $(PASSWORD)

$(SERVER_KEY_FILE): $(SERVER_PK12)
	openssl pkcs12 -in $< -nodes -nocerts -out $@ -passin pass:$(PASSWORD)

$(SERVER_CER_FILE): $(SERVER_PK12)
	openssl pkcs12 -in $< -nokeys -out $@ -passin pass:$(PASSWORD)

ALL=$(SERVER_PK12) $(SERVER_STORE) $(TRUST_STORE) $(SERVER_KEY_FILE) $(SERVER_CER_FILE)

secrets: $(ALL)
clean: 
	rm -f $(ALL)
