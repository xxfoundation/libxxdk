.PHONY: all



all:
	go build -buildmode=c-shared -o libxxdk.so .
	mkdir -p xxdk.NET/bin/Debug/net7.0/
	cp *.h *.so xxdk.NET/bin/Debug/net7.0/

