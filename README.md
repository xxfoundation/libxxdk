go build -buildmode=c-shared -o xxdk.so main.go


GOOS=darwin GOARCH=arm64 go build -trimpath -buildmode=c-shared -ldflags '-extldflags "-lresolv -undefined dynamic_lookup"' -o libxxdk.so main.go

// typedef long (* cmix_dm_receive_cb)(int dm_instance_id,
//    void* message_id, int message_id_len,
//    char* nickname, int nickname_len,
//    void* text, int text_len,
//    void* pubkey, int pubkey_len,
//    int dmToken, int codeset,
//    long timestamp, long round_id, long msg_type, long status);
//
