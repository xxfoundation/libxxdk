#ifndef TESTS_COMMON_H
#define TESTS_COMMON_H

#include "acutest.h"
#include "xxdk.h"

#include <filesystem>
#include <fstream>

namespace fs = std::filesystem;

static std::string read_file(const fs::path &path) {
  std::ifstream stream(path, std::ios::binary);
  TEST_ASSERT_(stream, "failed to open file %s", path.c_str());

  std::ostringstream str{};
  str << stream.rdbuf();
  TEST_ASSERT_(!stream.fail(), "failed to read file %s", path.c_str());

  return str.str();
}

static Cmix setup_test_instance(const fs::path &state_dir) {
  static const fs::path NDF_PATH{"tests-c/mainnet.json"};
  static const std::string PASSWORD{"testpass"};

  GoError err;

  auto stat = fs::status(state_dir);
  if (!fs::is_directory(stat)) {
    auto ndf = read_file(NDF_PATH);

    if (!TEST_CHECK(
            !(err = xx_NewCmix(ndf.c_str(), state_dir.c_str(), PASSWORD.data(),
                               PASSWORD.length(), "")))) {
      TEST_MSG("failed to create test Cmix storage dir %s: %s",
               state_dir.c_str(), err);
      free(err);
      fs::remove_all(state_dir);
      TEST_ASSERT(false);
    }
  }

  Cmix net;
  if (!TEST_CHECK(!(err = xx_LoadCmix(state_dir.c_str(), PASSWORD.data(),
                                      PASSWORD.length(), "", &net)))) {
    TEST_MSG("failed to load test Cmix from storage dir %s: %s",
             state_dir.c_str(), err);
    free(err);
    TEST_ASSERT(false);
  }

  return net;
}

#endif // TESTS_COMMON_H
