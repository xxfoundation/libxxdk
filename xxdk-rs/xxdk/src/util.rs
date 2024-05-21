use xxdk_sys::{GoByteSlice, GoError, GoSlice, GoString};

/// Copy the contents of a byte buffer into a Vec.
///
/// # Safety
///
/// `p` must point to an allocation of at least `n` bytes that is valid for the duration of this
/// function.
pub unsafe fn clone_bytes_from_raw_parts(p: *const u8, n: usize) -> Vec<u8> {
    let bytes = std::slice::from_raw_parts(p, n);
    Vec::from(bytes)
}

/// Copy the contents of a byte buffer into a String.
///
/// # Safety
///
/// `p` must point to an allocation of at least `n` bytes that is valid for the duration of this
/// function.
///
/// The memory pointed to by `p` must contain valid UTF-8.
pub unsafe fn clone_string_from_raw_parts(p: *const u8, n: usize) -> String {
    String::from_utf8_unchecked(clone_bytes_from_raw_parts(p, n))
}

/// Copy the contents of a C byte buffer into a Vec, and free the original allocation.
///
/// # Safety
///
/// The given slice must point to a valid C allocation. In particular, the slice must not be
/// a Go `nil` value.
///
/// The given slice must not be used (read or write) after this call returns.
pub unsafe fn c_byte_slice_into_vec(slice: GoByteSlice) -> Vec<u8> {
    let vec = clone_bytes_from_raw_parts(slice.data as *const u8, slice.len as usize);
    libc::free(slice.data);
    vec
}

/// Copy the contents of a byte slice into a new C buffer.
///
/// If the given slice is empty, no memory will be allocated and the returned buffer will be a null
/// pointer. Otherwise, the returned buffer will have been freshly allocated on the C heap.
pub fn clone_bytes_into_c_buffer(bytes: &[u8]) -> GoByteSlice {
    if bytes.is_empty() {
        GoByteSlice {
            len: 0,
            data: std::ptr::null_mut(),
        }
    } else {
        unsafe {
            let buf = libc::malloc(bytes.len() as libc::size_t);
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf as *mut u8, bytes.len());
            GoByteSlice {
                len: bytes.len() as libc::c_int,
                data: buf,
            }
        }
    }
}

/// Get a slice pointing to a statically-allocated Go string.
///
/// # Safety
///
/// `s.p` must point to a valid allocation of at least `s.n` bytes.
///
/// `s` must contain valid UTF-8. This should be the case for most strings returned from Go, but Go
/// does not guarantee this property.
///
/// The memory pointed to by `s` must be statically allocated and never garbage collected by Go.
pub unsafe fn static_go_string_as_str(s: GoString) -> &'static str {
    let bytes = std::slice::from_raw_parts(s.p as *const u8, s.n as usize);
    std::str::from_utf8_unchecked(bytes)
}

/// Construct a Go string referencing a Rust string slice.
///
/// This does not copy the string data; the returned Go string references the same memory as the
/// original string slice.
///
/// # Safety
///
/// The memory referenced by the given string slice must remain valid and immutable for the
/// lifetime of the returned GoString.
pub unsafe fn str_as_go_string(s: &str) -> GoString {
    bytes_as_go_string(s.as_bytes())
}

/// Construct a Go string referencing a Rust byte slice.
///
/// This does not copy the slice data; the returned Go string references the same memory as the
/// original byte slice.
///
/// # Safety
///
/// The memory referenced by the given byte slice must remain valid and immutable for the lifetime
/// of the returned GoString.
pub unsafe fn bytes_as_go_string(b: &[u8]) -> GoString {
    GoString {
        p: b.as_ptr() as *const i8,
        n: b.len() as isize,
    }
}

/// Construct a Go slice referencing a Rust byte slice.
///
/// This does not copy the slice data; the returned Go slice references the same memory as the
/// original byte slice.
///
/// # Safety
///
/// The memory referenced by the given byte slice must remain valid and immutable for the lifetime
/// of the returned GoSlice. In particular, the GoSlice must not be used to mutate the slice data.
pub unsafe fn bytes_as_go_slice(b: &[u8]) -> GoSlice {
    GoSlice {
        data: b.as_ptr() as *mut libc::c_void,
        len: b.len() as i64,
        cap: b.len() as i64,
    }
}

/// Convert a value/GoError pair into a Result, and free the original message buffer.
///
/// The `Ok` value is passed as a closure which is only evaluated if the error value is not an
/// error. This allows for cases in which evaluation of the `Ok` value is only safe in non-error
/// cases, e.g. dereferencing a pointer that is null in the case of an error.
///
/// # Safety
///
/// If `error.IsError` is nonzero, then `error.Msg` must point to a valid C allocation of at least
/// `error.MsgLen` bytes. The allocation must contain valid UTF-8, and it must not be used (read or
/// write) after this call returns.
///
/// If `error.IsError` is zero, then `error.Msg` must be null or dangling.
pub unsafe fn go_error_into_result<F, T>(val: F, error: GoError) -> Result<T, String>
where
    F: FnOnce() -> T,
{
    if error.IsError == 0 {
        Ok(val())
    } else {
        let s = clone_string_from_raw_parts(error.Msg as *const u8, error.MsgLen as usize);
        libc::free(error.Msg as *mut libc::c_void);
        Err(s)
    }
}
