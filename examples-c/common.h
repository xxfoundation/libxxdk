// Common functionality for C/C++ examples.

#ifndef _COMMON_H_
#define _COMMON_H_

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
bool write_file(const std::filesystem::path &path, const void *data, size_t len);

// Does the given path refer to a directory?
//
// Returns `true` if a file exists at the given path and it is a directory;
// returns `false` otherwise.
bool dir_exists(const std::filesystem::path &path);

#endif // _COMMON_H_
