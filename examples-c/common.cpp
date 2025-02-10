#include "common.h"
#include <cstddef>
#include <filesystem>
#include <fstream>
#include <sstream>

namespace fs = std::filesystem;

bool read_file(const std::filesystem::path &path, std::string &buf) {
  std::ifstream stream(path);
  if (stream) {
    buf.clear();
    std::ostringstream str;
    str.str().swap(buf);
    str << stream.rdbuf();
    str.str().swap(buf);
    return !stream.fail();
  }

  return false;
}

bool write_file(const fs::path &path, const void *data, size_t len) {
  std::ofstream stream(path, std::ios::binary);
  if (stream) {
    stream.write((const char *)data, len);
    return !stream.fail();
  }

  return false;
}

bool dir_exists(const fs::path &path) {
  auto stat = fs::status(path);
  return fs::is_directory(stat);
}
