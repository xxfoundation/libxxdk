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

#include "common.h"
#include "xxdk.h"
#include <cstring>
#include <filesystem>
#include <iostream>
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

const fs::path SENDER_CONTACT_PATH = "./myE2eContact.xxc";

int main() {
  GoError err = NULL;

  // Implemented in common.cpp
  Cmix net = load_cmix_state(STATE_PATH, SECRET, NDF_PATH, NDF_URL, CERT_PATH);

  // Load the reception identity, or create one if one doesn't already exist in
  // the client store.
  char *rid;
  if ((err = cmix_LoadReceptionIdentity(net, IDENTITY_STORAGE_KEY, &rid))) {
    if ((err = cmix_MakeReceptionIdentity(net, &rid))) {
      std::cerr << "Failed to create new reception identity: " << err
                << std::endl;
      free(rid);
      return -1;
    }

    if ((err = cmix_StoreReceptionIdentity(net, IDENTITY_STORAGE_KEY, rid))) {
      std::cerr << "Failed to store new reception identity: " << err
                << std::endl;
      free(rid);
      return -1;
    }
  }

  void *contact;
  int contact_len;
  if ((err = rid_GetContact(rid, &contact, &contact_len))) {
    std::cerr << "Failed to get contact info from reception identity: " << err
              << std::endl;
    free(rid);
    return -1;
  }

  if (!write_file(SENDER_CONTACT_PATH, (const void *)contact,
                  (size_t)contact_len)) {
    std::cerr << "Warning: failed to write contact file " << SENDER_CONTACT_PATH
              << std::endl;
  }

  free(contact);

  // TODO: Set up E2E client

  if ((err = cmix_StartNetworkFollower(net, 5000))) {
    std::cerr << "Failed to start network follower: " << err << std::endl;
    free(rid);
    return -1;
  }

  // TODO: Send messages

  free(rid);

  if ((err = cmix_StopNetworkFollower(net))) {
    std::cerr << "Failed to stop network follower: " << err << std::endl;
    return -1;
  }

  return 0;
}
