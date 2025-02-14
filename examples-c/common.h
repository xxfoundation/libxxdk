// Common functionality for C/C++ examples.

#ifndef _COMMON_H_
#define _COMMON_H_

#include "xxdk.h"
#include <cstddef>
#include <filesystem>
#include <string>

// Read the contents of the file at the given path into the given string.
//
// This will replace the contents of the string. Returns `true` on success and
// `false` on error.
bool read_file(const std::filesystem::path &path, std::string &buf);

// Write the contents of the given buffer to a file at the given path.
//
// Returns `true` on success, and `false` on failure.
bool write_file(const std::filesystem::path &path, const void *data,
                size_t len);

// Does the given path refer to a directory?
//
// Returns `true` if a file exists at the given path and it is a directory;
// returns `false` otherwise.
bool dir_exists(const std::filesystem::path &path);

// Initialize and load a Cmix instance state.
//
// This function encapsulates this common pattern seen throughout the xxDK
// examples:
//
// - Attempt to load an existing Cmix state directory.
//   - If the directory does not exist, attempt to load an NDF from a file.
//     - If the NDF does not exist locally, attempt to download and verify it.
//   - Initialize and load a new Cmix state directory.
// - Return the resulting Cmix instance.
//
// See `common.cpp` for the full code of this procedure.
//
// This function will exit the process if the state cannot be initialized or
// loaded. This is not a recommended error handling strategy for applications,
// but is expedient for examples.
Cmix load_cmix_state(const std::filesystem::path &state_path,
                     const std::string &state_password,
                     const std::filesystem::path &ndf_path,
                     const std::string &ndf_url,
                     const std::filesystem::path &cert_path);

#endif // _COMMON_H_
