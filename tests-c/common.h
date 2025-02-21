#ifndef TESTS_COMMON_H
#define TESTS_COMMON_H

#include <cstdio>
#include <cstdlib>
#include <filesystem>
#include <fstream>

#include "xxdk.h"

const static char *DEFAULT_NDF_PATH = "ndf.json";

static FILE *LOG_FILE;

static void init_xxdk_log(const char *test_name) {
  static const char *LOG_PREFIX = "ignore.log.";

  xx_DisableStdoutLog();

  std::string file_name{LOG_PREFIX};
  file_name += test_name;

  LOG_FILE = fopen(file_name.c_str(), "w");
  xx_SetLogFile(LOG_FILE);
}

static void fini_xxdk_log() {
  xx_ResetLogger();
  fclose(LOG_FILE);
}

#define TEST_INIT init_xxdk_log(test_name)
#define TEST_FINI fini_xxdk_log()

#include "acutest.h"

namespace fs = std::filesystem;

static std::string read_file(const fs::path &path) {
  std::ifstream stream(path, std::ios::binary);
  TEST_ASSERT_(stream, "failed to open file %s", path.c_str());

  std::ostringstream str{};
  str << stream.rdbuf();
  TEST_ASSERT_(!stream.fail(), "failed to read file %s", path.c_str());

  return str.str();
}

static fs::path get_test_ndf_path() {
  const char *env_path = getenv("XX_TEST_NDF_PATH");
  if (!env_path) {
    return fs::path{DEFAULT_NDF_PATH};
  }

  return fs::path{env_path};
}

static Cmix setup_test_instance(const fs::path &state_dir) {
  static const std::string PASSWORD{"testpass"};

  GoError err;

  auto stat = fs::status(state_dir);
  if (!fs::is_directory(stat)) {
    auto ndf = read_file(get_test_ndf_path());

    if (!TEST_CHECK(
            !(err = xx_NewCmix(ndf.c_str(), state_dir.c_str(), PASSWORD.data(),
                               PASSWORD.length(), "")))) {
      TEST_MSG("failed to create test Cmix storage dir %s: %s",
               state_dir.c_str(), err);
      fs::remove_all(state_dir);
      TEST_ASSERT(false);
    }
  }

  Cmix net;
  if (!TEST_CHECK(!(err = xx_LoadCmix(state_dir.c_str(), PASSWORD.data(),
                                      PASSWORD.length(), "", &net)))) {
    TEST_MSG("failed to load test Cmix from storage dir %s: %s",
             state_dir.c_str(), err);
    TEST_ASSERT(false);
  }

  return net;
}

#endif // TESTS_COMMON_H
