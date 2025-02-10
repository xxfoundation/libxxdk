#include "common.h"
#include "xxdk.h"
#include <filesystem>
#include <iostream>

namespace fs = std::filesystem;

const fs::path CONTACT_FILE_PATH = "./connectServer.xxc";
const fs::path STATE_PATH = "./statePath";
const std::string SECRET = "password";
const std::string NDF_URL = "https://elixxir-bins.s3.us-west-1.amazonaws.com/ndf/mainnet.json";
const fs::path CERT_PATH = "./mainnet.crt";
const fs::path NDF_PATH = "./mainnet.json";
const std::string IDENTITY_STORAGE_KEY = "identityStorageKey";

int main() {
  GoError err = NULL;

  if (!dir_exists(STATE_PATH)) {
    std::string ndf;

    if (!read_file(NDF_PATH, ndf)) {
      std::cerr << "Failed to read NDF file, attempting download..." << std::endl;

      std::string cert;
      if (!read_file(CERT_PATH, cert)) {
        std::cerr << "Failed to read certificate file" << std::endl;
        return -1;
      }

      char *downloaded_ndf;
      if ((err = xx_DownloadAndVerifySignedNdfWithUrl(NDF_URL.c_str(), cert.c_str(), &downloaded_ndf))) {
        std::cerr << "Failed to download NDF: " << err << std::endl;
        free(err);
        return -1;
      }

      ndf.assign(downloaded_ndf);
      free(downloaded_ndf);
    }

    if ((err = xx_NewCmix(ndf.c_str(), STATE_PATH.c_str(), (void *)SECRET.c_str(), SECRET.length(), ""))) {
      std::cerr << "Failed to initialize Cmix state: " << err << std::endl;
      free(err);
      fs::remove_all(STATE_PATH);
      return -1;
    }
  }

  Cmix net;
  if ((err = xx_LoadCmix(STATE_PATH.c_str(), (void *)SECRET.c_str(), SECRET.length(), "", &net))) {
    std::cerr << "Failed to load state: " << err << std::endl;
    free(err);
    return -1;
  }

  char *rid;
  if ((err = cmix_LoadReceptionIdentity(net, IDENTITY_STORAGE_KEY.c_str(), &rid))) {
    free(err);

    if ((err = cmix_MakeReceptionIdentity(net, &rid))) {
      std::cerr << "Failed to create new reception identity: " << err << std::endl;
      free(err);
      free(rid);
      return -1;
    }

    if ((err = cmix_StoreReceptionIdentity(net, IDENTITY_STORAGE_KEY.c_str(), rid))) {
      std::cerr << "Failed to store new reception identity: " << err << std::endl;
      free(err);
      free(rid);
      return -1;
    }
  }

  void *contact;
  int contact_len;
  if ((err = rid_GetContact(rid, &contact, &contact_len))) {
    std::cerr << "Failed to get contact info from reception identity: " << err << std::endl;
    free(err);
    free(rid);
    return -1;
  }

  if (!write_file(CONTACT_FILE_PATH.c_str(), contact, contact_len)) {
    std::cerr << "Warning: failed to write contact file " << CONTACT_FILE_PATH << std::endl;
  }

  free(contact);

  // TODO: Set up connect server

  if ((err = cmix_StartNetworkFollower(net, 5000))) {
    std::cerr << "Failed to start network follower: " << err << std::endl;
    free(err);
    free(rid);
    return -1;
  }

  // TODO: Handle connections

  free(rid);

  if ((err = cmix_StopNetworkFollower(net))) {
    std::cerr << "Failed to stop network follower: " << err << std::endl;
    free(err);
    return -1;
  }

  return 0;
}
