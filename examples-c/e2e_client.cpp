#include "libxxdk.h"
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <sstream>
#include <string>

namespace fs = std::filesystem;

const fs::path STATE_PATH = "./statePathRecipient";
const std::string NDF_URL =
    "https://elixxir-bins.s3.us-west-1.amazonaws.com/ndf/mainnet.json";
const fs::path CERT_PATH = "./mainnet.crt";
const fs::path NDF_PATH = "./mainnet.json";

const char *SECRET = "secret";

// Read the contents of the file at the given path into the given string.
//
// This will replace the contents of the string. Returns `true` on success and
// false` on error.
bool read_file(const fs::path &path, std::string &buf) {
  std::ifstream stream(path, std::ios::in | std::ios::binary);
  if (stream) {
    std::ostringstream str;
    str << stream.rdbuf();
    buf.assign(str.str());
    return true;
  }

  return false;
}

// Does the given path refer to a directory?
//
// Returns `false` if either there is no file at the given path, or if the file
// at the given path is not a directory.
bool dir_exists(const fs::path &path) {
  auto stat = fs::status(path);
  return fs::is_directory(stat);
}

int main() {
  GoError err;

  if (!dir_exists(STATE_PATH)) {
    std::string ndf;
    if (!read_file(NDF_PATH, ndf)) {
      std::cerr << "Failed to read NDF file, attempting to download..."
                << std::endl;

      std::string cert;
      if (!read_file(CERT_PATH, cert)) {
        std::cerr << "Failed to read certificate file" << std::endl;
        return -1;
      }

      char *downloaded_ndf;
      err = xx_DownloadAndVerifySignedNdfWithUrl(
          (char *)NDF_URL.c_str(), (char *)cert.c_str(), &downloaded_ndf);
      if (err) {
        std::cerr << "Failed to download NDF: " << err << std::endl;
        free(err);
        return -1;
      }

      std::cerr << "Downloaded NDF:\n" << downloaded_ndf << std::endl;
      ndf.assign(downloaded_ndf);
      free(downloaded_ndf);
    }

    err = xx_NewCmix((char *)ndf.c_str(), (char *)STATE_PATH.c_str(),
                     (void *)SECRET, strlen(SECRET), (char *)"");
    if (err) {
      std::cerr << "Failed to initialize state:" << err << std::endl;
      free(err);
      fs::remove_all(STATE_PATH);
      return -1;
    }
  }

  CMix net;
  err = xx_LoadCmix((char *)STATE_PATH.c_str(), (void *)SECRET, strlen(SECRET),
                    (char *)"", &net);
  if (err) {
    std::cerr << "Failed to load state: " << err << std::endl;
    free(err);
    return -1;
  }
}
