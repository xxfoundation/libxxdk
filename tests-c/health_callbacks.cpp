#include "acutest.h"
#include "common.h"
#include "xxdk.h"
#include <chrono>
#include <condition_variable>
#include <mutex>

typedef struct {
  std::mutex mu;
  std::condition_variable cv;
  int status;
} CallbackStatus;

void callback(int status, void *data) {
  CallbackStatus *cb_stat = (CallbackStatus *)data;

  std::unique_lock lk(cb_stat->mu);
  cb_stat->status = status;
  lk.unlock();
  cb_stat->cv.notify_all();
}

void test_callback() {
  const fs::path STATE_DIR{"ignore.test_callback"};

  Cmix net = setup_test_instance(STATE_DIR);

  GoError err = NULL;

  CallbackStatus cb_stat{std::mutex{}, std::condition_variable{}, 0};
  long cb_id;

  if (!TEST_CHECK(
          !(err = cmix_AddHealthCallback(net, callback, &cb_stat, &cb_id)))) {
    TEST_MSG("Failed to add health callback: %s", err);
    free(err);
    return;
  }

  if (!TEST_CHECK(!(err = cmix_StartNetworkFollower(net, 5000)))) {
    TEST_MSG("Failed to start network follower: %s", err);
    free(err);
    return;
  }

  {
    std::unique_lock lk(cb_stat.mu);
    auto now = std::chrono::system_clock::now();
    auto deadline = now + std::chrono::seconds{30};
    while (!cb_stat.status) {
      if (!TEST_CHECK(std::cv_status::no_timeout == cb_stat.cv.wait_until(lk, deadline))) {
        TEST_MSG("Timed out waiting for health callback");
        break;
      }
    }
  }

  if (!TEST_CHECK(!(err = cmix_RemoveHealthCallback(net, cb_id)))) {
    TEST_MSG("Failed to remove network health callback: %s", err);
    free(err);
  }

  if (!TEST_CHECK(!(err = cmix_StopNetworkFollower(net)))) {
    TEST_MSG("Failed to stop network follower: %s", err);
    free(err);
  }
}

TEST_LIST = {{"test_callback", test_callback}, {NULL, NULL}};
