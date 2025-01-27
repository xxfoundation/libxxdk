### Targets:
#
# all (default target): build shared library, header file, and all examples
# sharedlib: build xxDK shared library and header file
# examples-c: build all C examples
# clean: remove all build artifacts
#
### Configurable variables:
#
# - CXX (default "g++")
# 	C++ compiler
#
# - CPPFLAGS (default empty)
# 	C preprocessor flags
#
# - CXXFLAGS (default "-Wall -I. -I./sharedcgo")
# 	C++ compiler flags
#
# - LDFLAGS (default "-lxxdk -L. -Wl,-rpath,.")
# 	Linker flags
#
# - GOOS (default "linux")
# 	Cgo OS flag
#
# - GOARCH (default "amd64")
# 	Cgo architecture flag
#
# - LIBXXDK (default "libxxdk.dll" (windows) or "libxxdk.so" (others))
# 	File name for the xxDK shared library output by Cgo
#
# - LIBXXDK_H (default "libxxdk.h")
# 	File name for the xxDK header file output by Cgo

CXXFLAGS ?= -Wall -I. -I./sharedcgo
LDFLAGS ?= -lxxdk -L. -Wl,-rpath,.

GOOS ?= linux
GOARCH ?= amd64

# Name for the shared library; .dll on windows, .so on all other platforms
LIBXXDK_BASE := libxxdk
ifeq ($(GOOS), windows)
	LIBXXDK ?= $(LIBXXDK_BASE:=.dll)
else
	LIBXXDK ?= $(LIBXXDK_BASE:=.so)
endif

LIBXXDK_H ?= $(LIBXXDK_BASE:=.h)

EXAMPLES_C := e2e_client

# DOTNET = xxdk.NET

GODEPS := $(addprefix sharedcgo/,main.go callbacks.h callbacks.go rpc.go)

.PHONY: all sharedlib examples-c clean #windows-x64 windows-arm64 linux-x64 linux-arm64 darwin-x64 darwin-arm64 dotnet

all: sharedlib examples-c
sharedlib: $(LIBXXDK)

examples-c: $(EXAMPLES_C)

$(EXAMPLES_C): %: examples-c/%.o $(LIBXXDK)
	$(CXX) $(LDFLAGS) $< -o $@

%.o: %.cpp $(LIBXXDK:.so=.h)
	$(CXX) $(CPPFLAGS) $(CXXFLAGS) -c $< -o $@

$(LIBXXDK) $(LIBXXDK_H) &: $(GODEPS)
	CGO_ENABLED=1 GOOS=$(GOOS) GOARCH=$(GOARCH) go build -buildmode=c-shared -o $@ ./sharedcgo

clean:
	rm -f examples-c/*.o
	rm -f $(EXAMPLES_C)
	rm -f $(LIBXXDK) $(LIBXXDK_H)

# libxxdk-win-x64.dll: $(GODEPS)
# 	CGO_ENABLED=1 GOOS=windows GOARCH=amd64 go build -buildmode=c-shared -o $@ ./sharedcgo
# windows-x64: libxxdk-win-x64.dll
# 	mkdir -p $(DOTNET)/runtimes/win-x64/native
# 	cp *.h $(DOTNET)/runtimes/win-x64/native/
# 	cp libxxdk-win-x64.dll $(DOTNET)/runtimes/win-x64/native/xxdk.dll

# libxxdk-win-arm64.dll: $(GODEPS)
# 	CGO_ENABLED=1 GOOS=windows GOARCH=arm64 go build -buildmode=c-shared -o $@ ./sharedcgo
# windows-arm64: libxxdk-win-arm64.dll
# 	mkdir -p $(DOTNET)/runtimes/win-arm64/native
# 	cp *.h $(DOTNET)/runtimes/win-arm64/native/
# 	cp libxxdk-win-arm64.dll $(DOTNET)/runtimes/win-arm64/native/xxdk.dll

# libxxdk-linux-x64.so: $(GODEPS)
# 	CGO_ENABLED=1 GOOS=linux GOARCH=amd64 go build -buildmode=c-shared -o $@ ./sharedcgo
# linux-x64: libxxdk-linux-x64.so
# 	mkdir -p $(DOTNET)/runtimes/linux-x64/native
# 	cp *.h $(DOTNET)/runtimes/linux-x64/native/
# 	cp libxxdk-linux-x64.so $(DOTNET)/runtimes/linux-x64/native/libxxdk.so

# libxxdk-linux-arm64.so: $(GODEPS)
# 	CGO_ENABLED=1 GOOS=linux GOARCH=arm64 go build -buildmode=c-shared -o $@ ./sharedcgo
# linux-arm64: libxxdk-linux-arm64.so
# 	mkdir -p $(DOTNET)/runtimes/linux-arm64/native
# 	cp *.h $(DOTNET)/runtimes/linux-arm64/native/
# 	cp libxxdk-linux-arm64.so $(DOTNET)/runtimes/linux-arm64/native/libxxdk.so

# libxxdk-darwin-x64.so: $(GODEPS)
# 	CGO_ENABLED=1 GOOS=darwin GOARCH=amd64 go build -buildmode=c-shared -o $@ ./sharedcgo
# darwin-x64: libxxdk-darwin-x64.so
# 	mkdir -p $(DOTNET)/runtimes/darwin-x64/native
# 	cp *.h $(DOTNET)/runtimes/darwin-x64/native/
# 	cp libxxdk-darwin-x64.so $(DOTNET)/runtimes/darwin-x64/native/libxxdk.so

# libxxdk-darwin-arm64.so: $(GODEPS)
# 	CGO_ENABLED=1 GOOS=darwin GOARCH=arm64 go build -buildmode=c-shared -o $@ ./sharedcgo
# darwin-arm64: libxxdk-darwin-arm64.so
# 	mkdir -p $(DOTNET)/runtimes/darwin-arm64/native
# 	cp *.h $(DOTNET)/runtimes/darwin-arm64/native/
# 	cp libxxdk-darwin-arm64.so $(DOTNET)/runtimes/darwin-arm64/native/libxxdk.so

# dotnet:
# 	cd $(DOTNET) && dotnet build

# all: windows-x64 windows-arm64 linux-arm64 linux-arm64 darwin-x64 darwin-arm64 dotnet
#	mkdir -p xxdk.NET/bin/Debug/net7.0/
#	cp sharedcgo/*.h *.so xxdk.NET/bin/Debug/net7.0/
