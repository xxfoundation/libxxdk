#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod test {
    use std::slice;

    use super::*;

    unsafe fn gostring_to_string(s: GoString) -> String {
        let orig_bytes = slice::from_raw_parts(s.p as *const u8, s.n as usize);
        let bytes = Vec::from(orig_bytes);
        String::from_utf8_unchecked(bytes)
    }

    #[test]
    fn version() {
        let version_str = unsafe { gostring_to_string(GetVersion()) };
        println!("Version: {version_str}");
    }
}
