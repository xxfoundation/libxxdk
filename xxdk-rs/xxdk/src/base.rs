//! Base XXDK functions.
//!
//! This module provides safe Rust wrappers around the raw FFI bindings in `xxdk-sys`.
//!
//! Under normal circumstances, you should not need to use this module directly; prefer using the
//! high-level interfaces defined in [`xxdk::rpc`](crate::rpc) and [`xxdk::dm`](crate::dm).

use crate::util::*;
use xxdk_sys::*;

use std::sync::Arc;

pub mod dm;
pub mod rpc;

/// Get the dependencies string of the XXDK library.
pub fn get_dependencies() -> &'static str {
    unsafe { static_go_string_as_str(GetDependencies()) }
}

/// Get the version string of the XXDK library.
pub fn get_version() -> &'static str {
    unsafe { static_go_string_as_str(GetVersion()) }
}

/// Get the git version string of the XXDK library.
pub fn get_git_version() -> &'static str {
    unsafe { static_go_string_as_str(GetGitVersion()) }
}

/// A cMix instance.
#[derive(Debug)]
pub struct CMix {
    pub(crate) cmix_instance: i32,
}

impl CMix {
    /// Create a user storage, generate keys, and register with the network.
    ///
    /// Note that this does not register a username/identity, simply a cryptographic identity allowing
    /// the registration of such data at a later time.
    ///
    /// Users of this function should remove the storage directory on error.
    pub fn create(
        ndf_json: &str,
        storage_dir: &str,
        password: &[u8],
        registration_code: &str,
    ) -> Result<(), String> {
        // Need to clone this here, as mutable, since the password gets zeroed out on the Go
        // side.
        #[allow(unused_mut)]
        let mut password = Vec::from(password);
        unsafe {
            let err = NewCmix(
                str_as_go_string(ndf_json),
                str_as_go_string(storage_dir),
                bytes_as_go_slice(&password),
                str_as_go_string(registration_code),
            );
            go_error_into_result(|| (), err)
        }
    }

    /// Load an existing user storage.
    ///
    /// Note that loading more than one cMix instance with the same storage directory will result in
    /// data corruption.
    ///
    /// This function is non-blocking, and spawns subprocesses to handle network operations.
    ///
    /// # Errors
    ///
    /// Fails with an error if no user storage exists at the given file path, or if the given password
    /// is incorrect.
    pub fn load(storage_dir: &str, password: &[u8], params_json: &[u8]) -> Result<Self, String> {
        unsafe {
            let LoadCmix_return { r0, r1 } = LoadCmix(
                str_as_go_string(storage_dir),
                bytes_as_go_slice(password),
                bytes_as_go_slice(params_json),
            );
            go_error_into_result(|| CMix { cmix_instance: r0 }, r1)
        }
    }

    /// Load a user storage, generate keys, and register with the network.
    ///
    /// This creates the storage directory, generates keys, registers with the network, and
    /// loads the resulting cMix instance.
    ///
    /// If creation of the user storage fails, this will make an attempt to remove the created
    /// directory before returning an error.
    pub fn create_and_load(
        ndf_json: &str,
        storage_dir: &str,
        password: &[u8],
        registration_code: &str,
        params_json: &[u8],
    ) -> Result<Self, String> {
        if let Err(err) = Self::create(ndf_json, storage_dir, password, registration_code) {
            std::fs::remove_dir_all(storage_dir).ok();
            Err(err)
        } else {
            Self::load(storage_dir, password, params_json)
        }
    }

    /// Get the current default reception ID for this cMix instance.
    pub fn reception_id(&self) -> Result<Vec<u8>, String> {
        unsafe {
            let cmix_GetReceptionID_return { r0, r1 } = cmix_GetReceptionID(self.cmix_instance);
            go_error_into_result(|| c_byte_slice_into_vec(r0), r1)
        }
    }

    /// Get the value of a key in the KV store for this cMix instance.
    pub fn ekv_get(&self, key: &str) -> Result<Vec<u8>, String> {
        unsafe {
            let cmix_EKVGet_return { r0, r1 } =
                cmix_EKVGet(self.cmix_instance, str_as_go_string(key));
            go_error_into_result(|| c_byte_slice_into_vec(r0), r1)
        }
    }

    /// Set the value of a key in the KV store for this cMix instance.
    pub fn ekv_set(&self, key: &str, value: &[u8]) -> Result<(), String> {
        unsafe {
            go_error_into_result(
                || (),
                cmix_EKVSet(
                    self.cmix_instance,
                    str_as_go_string(key),
                    bytes_as_go_slice(value),
                ),
            )
        }
    }

    pub fn start_network_follower(&self, timeout_ms: i64) -> Result<(), String> {
        unsafe {
            go_error_into_result(
                || (),
                cmix_StartNetworkFollower(self.cmix_instance, timeout_ms),
            )
        }
    }

    pub fn stop_network_follower(&self) -> Result<(), String> {
        unsafe { go_error_into_result(|| (), cmix_StopNetworkFollower(self.cmix_instance)) }
    }

    pub fn wait_for_network(&self, timeout_ms: i64) -> Result<(), String> {
        unsafe { go_error_into_result(|| (), cmix_WaitForNetwork(self.cmix_instance, timeout_ms)) }
    }

    pub fn ready_to_send(&self) -> bool {
        unsafe { cmix_ReadyToSend(self.cmix_instance) != 0 }
    }
}

pub fn generate_codename_identity(passphrase: &str) -> Vec<u8> {
    // TODO: This function can panic from the Go side. Should investigate implications for
    // stack unwinding into Rust
    unsafe { c_byte_slice_into_vec(cmix_GenerateCodenameIdentity(str_as_go_string(passphrase))) }
}
