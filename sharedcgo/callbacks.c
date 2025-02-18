#include "callbacks.h"
#include <stdio.h>

size_t file_logger(void *file, const void *data, size_t data_len) {
  size_t n = fwrite(data, 1, data_len, (FILE *)file);
  if (n == data_len) {
    fflush((FILE *)file);
  }
  return n;
}
