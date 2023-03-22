.PHONY: all



all:
	go build -buildmode=c-shared -o libxxdk.so ./sharedcgo
	mkdir -p xxdk.NET/bin/Debug/net7.0/
	cp sharedcgo/*.h *.so xxdk.NET/bin/Debug/net7.0/

