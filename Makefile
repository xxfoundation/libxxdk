DOTNET = xxdk.NET

.PHONY: all windows-x64 windows-arm64 linux-arm64 linux-arm64 darwin-x64 darwin-arm64 dotnet

libxxdk-win-x64.dll:
	CGO_ENABLED=1 GOOS=windows GOARCH=amd64 go build -buildmode=c-shared -o $@ ./sharedcgo
windows-x64: libxxdk-win-x64.dll
	mkdir -p $(DOTNET)/runtimes/win-x64/native
	cp *.h libxxdk-win-x64.dll $(DOTNET)/runtimes/win-x64/native/

libxxdk-win-arm64.dll:
	CGO_ENABLED=1 GOOS=windows GOARCH=arm64 go build -buildmode=c-shared -o $@ ./sharedcgo
windows-arm64: libxxdk-win-arm64.dll
	mkdir -p $(DOTNET)/runtimes/win-arm64/native
	cp *.h libxxdk-win-arm64.dll $(DOTNET)/runtimes/win-arm64/native/

libxxdk-linux-x64.so:
	CGO_ENABLED=1 GOOS=linux GOARCH=amd64 go build -buildmode=c-shared -o $@ ./sharedcgo
linux-x64: libxxdk-linux-x64.so
	mkdir -p $(DOTNET)/runtimes/linux-x64/native
	cp *.h libxxdk-linux-x64.so $(DOTNET)/runtimes/linux-x64/native/

libxxdk-linux-arm64.so:
	CGO_ENABLED=1 GOOS=linux GOARCH=arm64 go build -buildmode=c-shared -o $@ ./sharedcgo
linux-arm64: libxxdk-linux-arm64.so
	mkdir -p $(DOTNET)/runtimes/linux-arm64/native
	cp *.h libxxdk-linux-arm64.so $(DOTNET)/runtimes/linux-arm64/native/

libxxdk-darwin-x64.so:
	CGO_ENABLED=1 GOOS=darwin GOARCH=amd64 go build -buildmode=c-shared -o $@ ./sharedcgo
darwin-x64: libxxdk-darwin-x64.so
	mkdir -p $(DOTNET)/runtimes/darwin-x64/native
	cp *.h libxxdk-darwin-x64.so $(DOTNET)/runtimes/darwin-x64/native/

libxxdk-darwin-arm64.so:
	CGO_ENABLED=1 GOOS=darwin GOARCH=amd64 go build -buildmode=c-shared -o $@ ./sharedcgo
darwin-arm64: libxxdk-darwin-arm64.so
	mkdir -p $(DOTNET)/runtimes/darwin-arm64/native
	cp *.h libxxdk-darwin-arm64.so $(DOTNET)/runtimes/darwin-arm64/native/

dotnet:
	cd $(DOTNET) && dotnet build

all: windows-x64 windows-arm64 linux-arm64 linux-arm64 darwin-x64 darwin-arm64 dotnet
#	mkdir -p xxdk.NET/bin/Debug/net7.0/
#	cp sharedcgo/*.h *.so xxdk.NET/bin/Debug/net7.0/
