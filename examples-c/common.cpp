#include "common.h"
#include "xxdk.h"
#include <cstddef>
#include <cstdlib>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>
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

Cmix load_cmix_state(const fs::path &state_path,
                     const std::string &state_password,
                     const fs::path &ndf_path, const std::string &ndf_url,
                     const fs::path &cert_path) {
  GoError err = NULL;

  // Create the state directory if it does not exist.
  if (!dir_exists(state_path)) {
    std::string ndf{};

    if (!read_file(ndf_path, ndf)) {
      // Download the NDF if it isn't available locally.
      std::cerr << "Failed to read NDF file, attempting download...\n";

      std::string cert{};
      if (!read_file(cert_path, cert)) {
        std::cerr << "Failed to read certificate file, exiting\n";
        std::exit(EXIT_FAILURE);
      }

      char *downloaded_ndf;
      if ((err = xx_DownloadAndVerifySignedNdfWithUrl(
               ndf_url.c_str(), cert.c_str(), &downloaded_ndf))) {
        std::cerr << "Failed to download NDF: " << err << std::endl;
        fs::remove_all(state_path);
        std::exit(EXIT_FAILURE);
      }

      ndf.assign(downloaded_ndf);
      free(downloaded_ndf);
    }

    if ((err =
             xx_NewCmix(ndf.c_str(), state_path.c_str(), state_password.data(),
                        state_password.length(), ""))) {
      std::cerr << "Failed to initialize new Cmix state: " << err << std::endl;
      fs::remove_all(state_path);
      std::exit(EXIT_FAILURE);
    }
  }

  // Load the Cmix client.
  Cmix net;
  if ((err = xx_LoadCmix(state_path.c_str(), state_password.data(),
                         state_password.length(), "", &net))) {
    std::cerr << "Failed to load Cmix state: " << err << std::endl;
    std::exit(EXIT_FAILURE);
  }

  return net;
}
