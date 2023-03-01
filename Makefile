.PHONY: all



all:
	go build -buildmode=c-shared -o libxxdk.so .
	cp *.h *.so xxdk.NET/bin/Debug/net7.0/

