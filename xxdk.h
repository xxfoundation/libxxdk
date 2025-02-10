#ifndef _XXDK_H_
#define _XXDK_H_

#ifdef __cplusplus
extern "C" {
#endif

// A Cmix instance ID.
typedef int Cmix;

// Error strings from the xxDK Go library.
//
// xxDK functions that may fail return `GoError`, and return their other results
// via out parameters.
//
// `GoError` is a typedef for `char *`. If a function returning `GoError` is
// successful, it will return null. Otherwise, it will return a null-terminated
// string representing a human-readable error message. This string will be
// allocated using `malloc`; the caller should arrange to free it.
typedef char *GoError;

// Network status callback functions.
//
// See [`cmix_AddHealthCallback`] for more information.
typedef void (*cmix_status_callback_fn)(int, void *);

// Get the xxDK version string.
//
// The string is allocated on the C heap with malloc. The caller should arrange
// for it to be freed.
char *xx_GetVersion();

// Get the xxDK git version string.
//
// The string is allocated on the C heap with malloc. The caller should arrange
// for it to be freed.
char *xx_GetGitVersion();

// Get the xxDK dependencies string.
//
// The string is allocated on the C heap with malloc. The caller should arrange
// for it to be freed.
char *xx_GetDependencies();

// Attempt to download an NDF from a specified URL.
//
// The NDF is processed into a protobuf containing a signature that is verified
// using the cert string passed in. The NDF is returned as JSON data that may be
// used to start a client.
GoError xx_DownloadAndVerifySignedNdfWithUrl(const char *url, const char *cert,
                                             char **out_ndf);

// Create a new cMix user storage.
//
// This will create the user storage directory, generate keys, connect,
// and register with the network. Note that this does not register a
// username/identity, but merely creates a new cryptographic identity for adding
// such information at a later date.
//
// Users of this function should delete the storage directory on error.
GoError xx_NewCmix(const char *ndfJSON, const char *storageDir,
                   const void *password, int passwordLen,
                   const char *registrationCode);

// Load an existing cMix user storage.
//
// This will fail if the user storage does not exist or the password is
// incorrect.
//
// The password is passed as a byte array so that it can be cleared from memory
// and stored as securely as possible using the MemGuard library.
//
// LoadCmix does not block on network connection and instead loads and starts
// subprocesses to perform network operations.
//
// This function returns, via the `out_cmix` parameter, a cMix Instance ID
// (int32) required to call specific cMix functions. If an error occurs,
// instance ID -1 is returned.
//
// Creating multiple cMix instance IDs with the same storage Dir will
// cause data corruption. In most cases only 1 instance should ever be
// needed.
GoError xx_LoadCmix(const char *storageDir, const void *password,
                    int passwordLen, const char *cmixParamsJSON,
                    Cmix *out_cmix);

// Get the current default reception ID for the given cMix instance.
//
// On success, `*out_rid` will contain the reception ID as a null-terminated
// JSON string. The reception ID is allocated using `malloc`; the caller should
// arrange to free it.
//
// On error, `*out_rid` will be null.
GoError cmix_GetReceptionID(Cmix cmix, char **out_rid);

// Generate a new cryptographic identity for receiving messages.
//
// The reception identity is returned as a null-terminated JSON string allocated
// with `malloc`. On error, `*out_rid` will be null.
GoError cmix_MakeReceptionIdentity(Cmix cmix, char **out_rid);

// Store the given reception identity in the given Cmix instance's encrypted
// key-value store, with the given key.
GoError cmix_StoreReceptionIdentity(Cmix cmix, const char *key,
                                    const char *rid);

// Load a reception identity from the given Cmix instance's encrypted store,
// using the given key.
//
// The reception identity is returned as a null-terminated JSON string allocated
// with `malloc`. On error, `*out_rid` will be null.
GoError cmix_LoadReceptionIdentity(Cmix cmix, const char *key, char **out_rid);

// Retrieve a value inside the given cMix instance's encrypted key-value store.
//
// On success, `*out_value` will contain a pointer to the retrieved value, and
// `*out_valueLen` will contain the length in bytes of the value. The value is
// allocated using `malloc`; the caller should arrange to free it.
//
// On error, `*out_value` will be null, and `*out_valueLen` will be 0.
GoError cmix_EKVGet(Cmix cMix, const char *key, void **out_value,
                    int *out_valueLen);

// Set a value inside the given cMix instance's encrypted key-value store.
GoError cmix_EKVSet(Cmix cMix, const char *key, void *value, int valueLen);

// Start network tracking threads for the given Cmix instance.
//
// If the network tracking threads for this instance are currently in the
// process of stopping, this will wait for up to the given timeout for them to
// become fully stopped before reststarting them. If the timeout elapses before
// the tracking threads are fully stopped, this will return an error.
GoError cmix_StartNetworkFollower(Cmix cmix, int timeoutMS);

// Stop network tracking threads for the given Cmix instance.
//
// This will return an error if the tracking threads are in the wrong state to
// be stopped, or if they fail to stop.
GoError cmix_StopNetworkFollower(Cmix cmix);

// Block until either the network is healthy or the given timeout elapses.
GoError cmix_WaitForNetwork(Cmix cmix, int timeoutMS);

// Add a callback to the given Cmix instance to be called whenever the network
// health status changes.
//
// The callback will be called with the following arguments:
// - An integer indicating the network health status: 1 for healthy and 0
//   otherwise.
// - The value of the data pointer given here.
//
// This callback/data pair will be assigned an integer ID within the given Cmix
// instance, returned via `out_id`, which can later be used to remove the
// callback from the Cmix instance.
GoError cmix_AddHealthCallback(Cmix cmix, cmix_status_callback_fn cb,
                               void *data, long *out_id);

// Remove a health status callback from the given Cmix instance.
//
// `id` is the integer ID returned by `cmix_AddHealthCallback`.
GoError cmix_RemoveHealthCallback(Cmix cmix, long id);

// Is this Cmix instance ready to send messages?
//
// This checks if the network is healthy, and if the Cmix client is registered
// with at least 70% of the network nodes.
//
// Returns 1 if the instance is ready to send, and 0 otherwise.
int cmix_ReadyToSend(Cmix cmix);

// Get the contact for the given reception identity.
//
// The contact is returned as a `malloc`-allocated byte array in a compact
// binary format. On error, `*out_contact` will be null and `*out_contactLen`
// will be zero.
GoError rid_GetContact(const char *rid, void **out_contact,
                       int *out_contactLen);

#ifdef __cplusplus
} // extern "C"
#endif

#endif // _XXDK_H_
