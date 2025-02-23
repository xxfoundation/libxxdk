// C implementations of necessary library functionality.

// ========================================================
// Thread-local info string implementation (library info strings, error strings)

#include "c_impl.h"
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <threads.h>

typedef struct {
  char *buf;
  size_t alloc;
} XxInfoString;

// The thread-local info string struct itself.
thread_local XxInfoString INFO_STRING = {NULL, 0};

// TSS key (shared across threads, needed in order to register destructor function)
static tss_t INFO_STRING_KEY;

// Call-once flag for initializing INFO_STRING_KEY.
static once_flag INFO_STRING_KEY_INIT_FLAG = ONCE_FLAG_INIT;

// Info string TSS destructor.
void free_info_string(void *ptr) {
  if (!ptr) {
    return;
  }

  XxInfoString *info_string = (XxInfoString *)ptr;
  if (info_string->buf != NULL) {
    free(info_string->buf);
  }

  info_string->buf = NULL;
  info_string->alloc = 0;

  // Zero out the TSS key so the destructor won't re-run
  tss_set(INFO_STRING_KEY, NULL);
}

// Called-once function to initialize INFO_STRING_KEY.
void create_info_string_key() {
  tss_create(&INFO_STRING_KEY, free_info_string);
}

// Lazily initialize the thread's info string.
//
// This will initialize INFO_STRING_KEY if it hasn't been initialized yet, and
// set the current thread's value for that key to point to its local info string
// if needed.
void init_info_string() {
  call_once(&INFO_STRING_KEY_INIT_FLAG, create_info_string_key);
  if (!tss_get(INFO_STRING_KEY)) {
    tss_set(INFO_STRING_KEY, &INFO_STRING);
  }
}

const char *set_info_string(const void *contents, size_t new_len) {
  init_info_string();

  if (contents) {
    // The given len is the length of the string not including the terminating
    // null byte.
    size_t new_alloc = new_len + 1;

    if (INFO_STRING.alloc < new_alloc) {
      char *new_buf = realloc(INFO_STRING.buf, new_alloc);
      if (!new_buf) {
        fprintf(stderr, "Out of memory");
        abort();
      }
      INFO_STRING.buf = new_buf;
      INFO_STRING.alloc = new_alloc;
    }

    memcpy(INFO_STRING.buf, contents, new_len);
    INFO_STRING.buf[new_len] = '\0';
  }

  return INFO_STRING.buf;
}


// ========================================================
// Logging configuration

size_t bridge_log_output(xx_log_output_fn logger, void *logger_data, const void *message, size_t message_len) {
	return logger(logger_data, message, message_len);
}

size_t file_logger(void *file, const void *data, size_t data_len) {
  size_t n = fwrite(data, 1, data_len, (FILE *)file);
  if (n == data_len) {
    fflush((FILE *)file);
  }
  return n;
}

// ========================================================
// Network health callbacks

void bridge_health_callback(cmix_status_callback_fn cb, int healthy, void *cb_data) {
	cb(healthy, cb_data);
}
