#include "acutest.h"
#include "common.h"
#include "xxdk.h"
#include <cstring>

static Cmix net;

void raw_ekv_roundtrip() {
  const char *data = "Testing EKV roundtrip";
  const char *key = "testEKVKey";

  GoError err = NULL;

  if (!TEST_CHECK(
          !(err = cmix_EKVSet(net, key, (void *)data, strlen(data) + 1)))) {
    TEST_MSG("Failed to set EKV value: %s", err);
    free(err);
    return;
  }

  char *out_data = NULL;
  int out_data_len = 0;

  if (!TEST_CHECK(
          !(err = cmix_EKVGet(net, key, (void **)&out_data, &out_data_len)))) {
    TEST_MSG("Failed to get EKV value: %s", err);
    free(err);
    return;
  }

  if (!TEST_CHECK((size_t)out_data_len == strlen(data) + 1)) {
    TEST_MSG(
        "Retrieved EKV value len (%d) does not match stored data len (%zu)",
        out_data_len, strlen(data) + 1);
    free(out_data);
    return;
  }

  TEST_CHECK(strcmp(data, out_data) == 0);
  TEST_MSG("Retrieved EKV value (\"%s\") does not match stored value (\"%s\")",
           out_data, data);

  free(out_data);
}

void rid_ekv_roundtrip() {
  const char *key = "testRIDKey";

  GoError err = NULL;

  char *rid;

  if (!TEST_CHECK(!(err = cmix_MakeReceptionIdentity(net, &rid)))) {
    TEST_MSG("Failed to create RID: %s", err);
    free(err);
    return;
  }

  if (!TEST_CHECK(!(err = cmix_StoreReceptionIdentity(net, key, rid)))) {
    TEST_MSG("Failed to store RID: %s", err);
    free(err);
    free(rid);
    return;
  }

  char *out_rid;
  if (!TEST_CHECK(!(err = cmix_LoadReceptionIdentity(net, key, &out_rid)))) {
    TEST_MSG("Failed to load RID: %s", err);
    free(err);
    free(rid);
    return;
  }

  TEST_CHECK(strcmp(rid, out_rid) == 0);
  TEST_MSG("Retrieved RID (%s) does not match stored (%s)", out_rid, rid);

  free(rid);
  free(out_rid);
}

void ekv_tests() {
  TEST_CASE("ekv_tests_setup");

  const fs::path STATE_DIR{"ignore.ekv"};
  net = setup_test_instance(STATE_DIR);

  TEST_CASE("raw_ekv_roundtrip");
  raw_ekv_roundtrip();

  TEST_CASE("rid_ekv_roundtrip");
  rid_ekv_roundtrip();
}

TEST_LIST = {{"ekv_tests", ekv_tests}, {NULL, NULL}};
