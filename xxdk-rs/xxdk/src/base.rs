//! Base XXDK functions.
//!
//! This module provides safe Rust wrappers around the raw FFI bindings in `xxdk-sys`.

use crate::util::*;

use xxdk_sys::*;

pub mod callbacks;

// Safety for get_dependencies, get_version, and get_git_version:
//
// - These functions return static global Go strings that are never garbage collected.
// - The returned strings are defined as string literals in the Go source, and so are guaranteed to
//   be valid UTF-8.

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

/// Create a user storage, generate keys, and register with the network.
///
/// Note that this does not register a username/identity, simply a cryptographic identity allowing
/// the registration of such data at a later time.
///
/// Users of this function should remove the storage directory on error.
pub fn new_cmix(
    ndf_json: &str,
    storage_dir: &str,
    password: &[u8],
    registration_code: &str,
) -> Result<(), String> {
    unsafe {
        let err = NewCmix(
            str_as_go_string(ndf_json),
            str_as_go_string(storage_dir),
            bytes_as_go_slice(password),
            str_as_go_string(registration_code),
        );
        go_error_into_result(|| (), err)
    }
}

/// Load an existing user storage.
///
/// Returns a cMix instance ID.
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
pub fn load_cmix(storage_dir: &str, password: &[u8], params_json: &[u8]) -> Result<i32, String> {
    unsafe {
        let LoadCmix_return { r0, r1 } = LoadCmix(
            str_as_go_string(storage_dir),
            bytes_as_go_slice(password),
            bytes_as_go_slice(params_json),
        );
        go_error_into_result(|| r0, r1)
    }
}

pub mod cmix {
    use super::*;

    /// Get the current default reception ID for the given cMix instance.
    pub fn get_reception_id(cmix_instance: i32) -> Result<Vec<u8>, String> {
        unsafe {
            let cmix_GetReceptionID_return { r0, r1 } = cmix_GetReceptionID(cmix_instance);
            go_error_into_result(|| c_byte_slice_into_vec(r0), r1)
        }
    }

    /// Get the value of a key in the KV store for the given cMix instance.
    pub fn ekv_get(cmix_instance: i32, key: &str) -> Result<Vec<u8>, String> {
        unsafe {
            let cmix_EKVGet_return { r0, r1 } = cmix_EKVGet(cmix_instance, str_as_go_string(key));
            go_error_into_result(|| c_byte_slice_into_vec(r0), r1)
        }
    }

    /// Set the value of a key in the KV store for the given cMix instance.
    pub fn ekv_set(cmix_instance: i32, key: &str, value: &[u8]) -> Result<(), String> {
        unsafe {
            go_error_into_result(
                || (),
                cmix_EKVSet(
                    cmix_instance,
                    str_as_go_string(key),
                    bytes_as_go_slice(value),
                ),
            )
        }
    }

    pub fn start_network_follower(cmix_instance: i32, timeout_ms: i64) -> Result<(), String> {
        unsafe { go_error_into_result(|| (), cmix_StartNetworkFollower(cmix_instance, timeout_ms)) }
    }

    pub fn stop_network_follower(cmix_instance: i32) -> Result<(), String> {
        unsafe { go_error_into_result(|| (), cmix_StopNetworkFollower(cmix_instance)) }
    }

    pub fn wait_for_network(cmix_instance: i32, timeout_ms: i64) -> Result<(), String> {
        unsafe { go_error_into_result(|| (), cmix_WaitForNetwork(cmix_instance, timeout_ms)) }
    }

    pub fn ready_to_send(cmix_instance: i32) -> bool {
        unsafe { cmix_ReadyToSend(cmix_instance) != 0 }
    }

    pub fn generate_codename_identity(passphrase: &str) -> Vec<u8> {
        // TODO: This function can panic from the Go side. Should investigate implications for
        // stack unwinding into Rust
        unsafe {
            c_byte_slice_into_vec(cmix_GenerateCodenameIdentity(str_as_go_string(passphrase)))
        }
    }

    pub mod dm {
        use super::*;

        pub fn new_dm_client(
            cmix_instance: i32,
            codename_identity: &[u8],
            passphrase: &str,
        ) -> Result<i32, String> {
            unsafe {
                let cmix_dm_NewDMClient_return { r0, r1 } = cmix_dm_NewDMClient(
                    cmix_instance,
                    bytes_as_go_slice(codename_identity),
                    str_as_go_string(passphrase),
                );
                go_error_into_result(|| r0, r1)
            }
        }

        pub fn get_dm_token(dm_instance: i32) -> Result<i32, String> {
            unsafe {
                let cmix_dm_GetDMToken_return { r0, r1 } = cmix_dm_GetDMToken(dm_instance);
                go_error_into_result(|| r0, r1)
            }
        }

        pub fn get_dm_pubkey(dm_instance: i32) -> Result<Vec<u8>, String> {
            unsafe {
                let cmix_dm_GetDMPubKey_return { r0, r1 } = cmix_dm_GetDMPubKey(dm_instance);
                go_error_into_result(|| c_byte_slice_into_vec(r0), r1)
            }
        }

        pub fn send(
            dm_instance: i32,
            partner_pubkey: &[u8],
            dm_token: i32,
            message_type: i64,
            plaintext: &[u8],
            lease_time_ms: i64,
            cmix_params_json: &[u8],
        ) -> Result<Vec<u8>, String> {
            unsafe {
                let cmix_dm_Send_return { r0, r1 } = cmix_dm_Send(
                    dm_instance,
                    bytes_as_go_slice(partner_pubkey),
                    dm_token,
                    message_type,
                    bytes_as_go_slice(plaintext),
                    lease_time_ms,
                    bytes_as_go_slice(cmix_params_json),
                );
                go_error_into_result(|| c_byte_slice_into_vec(r0), r1)
            }
        }

        pub fn send_text(
            dm_instance: i32,
            partner_pubkey: &[u8],
            dm_token: i32,
            message: &str,
            lease_time_ms: i64,
            cmix_params_json: &[u8],
        ) -> Result<Vec<u8>, String> {
            unsafe {
                let cmix_dm_SendText_return { r0, r1 } = cmix_dm_SendText(
                    dm_instance,
                    bytes_as_go_slice(partner_pubkey),
                    dm_token,
                    str_as_go_string(message),
                    lease_time_ms,
                    bytes_as_go_slice(cmix_params_json),
                );
                go_error_into_result(|| c_byte_slice_into_vec(r0), r1)
            }
        }

        pub fn send_reply(
            dm_instance: i32,
            partner_pubkey: &[u8],
            dm_token: i32,
            message: &str,
            reply_to: &[u8],
            lease_time_ms: i64,
            cmix_params_json: &[u8],
        ) -> Result<Vec<u8>, String> {
            unsafe {
                let cmix_dm_SendReply_return { r0, r1 } = cmix_dm_SendReply(
                    dm_instance,
                    bytes_as_go_slice(partner_pubkey),
                    dm_token,
                    str_as_go_string(message),
                    bytes_as_go_slice(reply_to),
                    lease_time_ms,
                    bytes_as_go_slice(cmix_params_json),
                );
                go_error_into_result(|| c_byte_slice_into_vec(r0), r1)
            }
        }

        pub fn send_reaction(
            dm_instance: i32,
            partner_pubkey: &[u8],
            dm_token: i32,
            message: &str,
            react_to: &[u8],
            lease_time_ms: i64,
            cmix_params_json: &[u8],
        ) -> Result<Vec<u8>, String> {
            unsafe {
                let cmix_dm_SendReaction_return { r0, r1 } = cmix_dm_SendReaction(
                    dm_instance,
                    bytes_as_go_slice(partner_pubkey),
                    dm_token,
                    str_as_go_string(message),
                    bytes_as_go_slice(react_to),
                    lease_time_ms,
                    bytes_as_go_slice(cmix_params_json),
                );
                go_error_into_result(|| c_byte_slice_into_vec(r0), r1)
            }
        }
    }
}
