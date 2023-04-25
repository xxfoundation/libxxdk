This is a sample library showing xxDK running inside of a .NET runtime
with a simple C# library wrapper.

Running and Testing
===================

This is tested to run on MacOS (M1/Arm version) and Linux (Ubuntu
x86_64). It should run on Windows but may need the library name
changed.

```
make -k
cd xxdk.NET
dotnet run --ndf mainnet-ndf.json --state-dir world --wait 20 | grep ^DM
```

NOTE: you may need to specify a compiler to the make command, especially
when cross compiling. Example for compiling to windows:
```
CC=x86_64-w64-mingw32-gcc make -k
cd xxdk.NET
dotnet run --ndf mainnet-ndf.json --state-dir world --wait 20 | grep ^DM
```


It's highly recommended to run with grep on the output. Especially on
mainnet, logs can be noisy with failure to connect errors as it accesses
the network.

Once you've run once with dotnet (or built it), you can run the
`dm.sh` shell script to have 2 clients talk to each other (or you can
run it on your own).

Main logic is in Program.cs, the xxdk library wrapper is in xxdk.cs.



Manual Build Commands:
======================

Generic:
```
go build -buildmode=c-shared -o xxdk.so main.go
```

More complex example (which would support direct extern C callbacks):

```
GOOS=darwin GOARCH=arm64 go build -trimpath -buildmode=c-shared -ldflags '-extldflags "-lresolv -undefined dynamic_lookup"' -o libxxdk.so main.go
```

We moved the Callbacks into a wrapper, so the simple one will work for
everyone now.
