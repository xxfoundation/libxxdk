This is a sample library showing xxDK running inside of a .NET runtime
with a simple C# library wrapper.

Running and Testing
===================

```
make -k
cd xxdk.NET
dotnet run --ndf mainnet-ndf.json --state-dir world | grep ^DM
```

It's highly recommended to run with grep on the output. Especially on
mainnet, logs can be noisy with failure to connect errors as it accesses
the network.

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
