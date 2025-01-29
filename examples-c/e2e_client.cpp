// E2E Client xxDK example
//
// This is a C++ translation of the E2E client example:
// https://git.xx.network/xx_network/xxdk-examples/-/tree/master/other-examples/e2eClient
//
// It makes use of C++17 features, in particular the standard library
// cross-platform filesystem API. Your C++ compiler must support C++17 in order
// to build this example.
//
// To build:
//
//   $ make e2e_client
//
// To run:
//
//   $ ./e2e_client

#include "libxxdk.h"
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <sstream>
#include <string>

namespace fs = std::filesystem;

// Path to the cMix client state directory.
const fs::path STATE_PATH = "./statePathRecipient";

// State directory password.
const char *SECRET = "secret";

// Reception identity storage key.
const char *IDENTITY_STORAGE_KEY = "identityStorageKey";

// Path to a local NDF.
const fs::path NDF_PATH = "./mainnet.json";

// URL from which to download the NDF if the local file is not available.
const std::string NDF_URL =
    "https://elixxir-bins.s3.us-west-1.amazonaws.com/ndf/mainnet.json";

// Certificate for the online NDF.
const fs::path CERT_PATH = "./mainnet.crt";

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
  GoError err = NULL;

  // Create the state directory if it does not exist.
  if (!dir_exists(STATE_PATH)) {
    std::string ndf;

    // Download the NDF if it's not available.
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

  // Load the cMix client.
  Cmix net;
  err = xx_LoadCmix((char *)STATE_PATH.c_str(), (void *)SECRET, strlen(SECRET),
                    (char *)"", &net);
  if (err) {
    std::cerr << "Failed to load state: " << err << std::endl;
    free(err);
    return -1;
  }

  // Load the reception identity, or create one if one doesn't already exist in
  // the client store.
  char *rid;
  if ((err = cmix_LoadReceptionIdentity(net, (char *)IDENTITY_STORAGE_KEY,
                                        &rid))) {
    free(err);

    if ((err = cmix_MakeReceptionIdentity(net, &rid))) {
      std::cerr << "Failed to create new reception identity: " << err
                << std::endl;
      free(err);
      return -1;
    }

    if ((err = cmix_StoreReceptionIdentity(net, (char *)IDENTITY_STORAGE_KEY,
                                           rid))) {
      std::cerr << "Failed to store new reception identity: " << err
                << std::endl;
      free(err);
      return -1;
    }
  }

  std::cout << "Reception ID: " << rid << std::endl;
  void *contact;
  int contact_len;
  if ((err = rid_GetContact(rid, &contact, &contact_len))) {
    std::cerr << "Failed to get contact info from reception identity: " << err << std::endl;
    free(err);
    return -1;
  }

  free(rid);
  free(contact);
  if (err) {
    free(err);
  }

  return 0;
}
