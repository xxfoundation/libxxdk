#ifndef _XXDK_H_
#define _XXDK_H_

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

// A Cmix instance ID.
typedef int Cmix;

// Error strings from the xxDK Go library.
//
// xxDK functions that may fail return `GoError`, and return their other results
// via out parameters.
//
// `GoError` is a typedef for `const char *`. If a function returning `GoError`
// is successful, it will return NULL. Otherwise, it will return a
// null-terminated string representing a human-readable error message. This
// string is allocated in thread-local storage managed by xxDK, and will be
// overwritten by subsequent error strings or calls to `xx_GetVersion()`,
// `xx_GetGitVersion()`, or `xx_GetDependencies()`. It should not be retained,
// modified, or freed by the user.
typedef const char *GoError;

// Network status callback functions.
//
// See `cmix_AddHealthCallback` for more information.
typedef void (*cmix_status_callback_fn)(int networkStatus, void *userData);

// xxDK logging output callback functions.
//
// See `xx_SetLogger` for more information.
typedef size_t (*xx_log_output_fn)(void *loggerData, const void *data,
                                   size_t dataLen);

// Set the xxDK log output callback.
//
// By default, xxDK will write log output to standard output. It can
// additionally write it to a user-configured logging system by means of this
// function.
//
// This will replace any log output previously set by either `xx_SetLogger` or
// `xx_SetLogFile`. If logs should be written to multiple outputs, the callback
// set here must handle each output.
//
// This function is intended to allow integration of xxDK logs into other
// logging systems. To simply write log output to a file, see `xx_SetLogFile`.
//
// For each message logged by xxDK, `loggerFn` will be called with the
// following arguments:
//
// - The `loggerData` pointer given here;
// - A pointer to the bytes of the logged message;
// - The byte length of the logged message.
//
// The callback must not retain the log message pointer, nor modify the memory
// to which it points.
//
// If `loggerFn` accesses `loggerData`, then `loggerData` should remain valid
// until either the process exits or `xx_ResetLogger` is called to clear the
// configured logger.
void xx_SetLogger(xx_log_output_fn loggerFn, void *loggerData);

// Set the xxDK log file.
//
// By default, xxDK will write log output to standard output. To additionally
// write it to a file, use this function.
//
// This will replace any log output previously set by either `xx_SetLogger` or
// `xx_SetLogFile`. If logs should be written to multiple outputs, use
// `xx_SetLogger` with a callback that handles each output.
//
// The given file must be opened for writing, and must remain valid until either
// the process exits or `xx_ResetLogger` is called to clear the configured
// logger.
void xx_SetLogFile(FILE *file);

// Clear the configured xxDK log output.
//
// This will clear any log output previously configured by either `xx_SetLogger`
// or `xx_SetLogFile`.
void xx_ResetLogger();

// Disable writing of xxDK logs to standard output.
void xx_DisableStdoutLog();

// Enable writing of xxDK logs to standard output.
//
// This is the default state of the library when loaded.
void xx_EnableStdoutLog();

// Get the xxDK version string.
//
// The string is allocated in thread-local storage managed by xxDK, and may be
// overwritten or reallocated by subsequent calls to `xx_GetVersion()`,
// `xx_GetGitVersion()`, `xx_GetDependencies()`, or any function that returns a
// `GoError`. It should not be retained, modified, or freed by the user.
const char *xx_GetVersion();

// Get the xxDK git version string.
//
// The string is allocated in thread-local storage managed by xxDK, and may be
// overwritten or reallocated by subsequent calls to `xx_GetVersion()`,
// `xx_GetGitVersion()`, `xx_GetDependencies()`, or any function that returns a
// `GoError`. It should not be retained, modified, or freed by the user.
const char *xx_GetGitVersion();

// Get the xxDK dependencies string.
//
// The string is allocated in thread-local storage managed by xxDK, and may be
// overwritten or reallocated by subsequent calls to `xx_GetVersion()`,
// `xx_GetGitVersion()`, `xx_GetDependencies()`, or any function that returns a
// `GoError`. It should not be retained, modified, or freed by the user.
const char *xx_GetDependencies();

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
