go build -buildmode=c-shared -o xxdk.so main.go


GOOS=darwin GOARCH=arm64 go build -trimpath -buildmode=c-shared -ldflags '-extldflags "-lresolv -undefined dynamic_lookup"' -o libxxdk.so main.go



Running it:

make -k
cd xxdk.NET
dotnet run --ndf mainnet-ndf.json --state-dir world
