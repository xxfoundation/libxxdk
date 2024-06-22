//! Base XXDK functions.
//!
//! This module provides safe Rust wrappers around the raw FFI bindings in `xxdk-sys`.

use crate::util::*;
use xxdk_sys::*;

use std::sync::Arc;

pub mod callbacks;

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

use callbacks::DmCallbacks;

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

    pub fn new_dm_client(
        &self,
        codename_identity: &[u8],
        passphrase: &str,
        callbacks: Arc<dyn DmCallbacks>,
    ) -> Result<Dm, String> {
        let instance_id = unsafe {
            let cmix_dm_NewDMClient_return { r0, r1 } = cmix_dm_NewDMClient(
                self.cmix_instance,
                bytes_as_go_slice(codename_identity),
                str_as_go_string(passphrase),
            );
            go_error_into_result(|| r0, r1)?
        };

        let dm = Dm { instance_id };
        dm.set_callbacks(callbacks);
        Ok(dm)
    }
}

pub fn generate_codename_identity(passphrase: &str) -> Vec<u8> {
    // TODO: This function can panic from the Go side. Should investigate implications for
    // stack unwinding into Rust
    unsafe { c_byte_slice_into_vec(cmix_GenerateCodenameIdentity(str_as_go_string(passphrase))) }
}

#[derive(Debug)]
pub struct Dm {
    pub(crate) instance_id: i32,
}

impl Dm {
    pub fn get_token(&self) -> Result<i32, String> {
        unsafe {
            let cmix_dm_GetDMToken_return { r0, r1 } = cmix_dm_GetDMToken(self.instance_id);
            go_error_into_result(|| r0, r1)
        }
    }

    pub fn get_dm_pubkey(&self) -> Result<Vec<u8>, String> {
        unsafe {
            let cmix_dm_GetDMPubKey_return { r0, r1 } = cmix_dm_GetDMPubKey(self.instance_id);
            go_error_into_result(|| c_byte_slice_into_vec(r0), r1)
        }
    }

    pub fn send(
        &self,
        partner_pubkey: &[u8],
        dm_token: i32,
        message_type: i64,
        plaintext: &[u8],
        lease_time_ms: i64,
        cmix_params_json: &[u8],
    ) -> Result<Vec<u8>, String> {
        unsafe {
            let cmix_dm_Send_return { r0, r1 } = cmix_dm_Send(
                self.instance_id,
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
        &self,
        partner_pubkey: &[u8],
        dm_token: i32,
        message: &str,
        lease_time_ms: i64,
        cmix_params_json: &[u8],
    ) -> Result<Vec<u8>, String> {
        unsafe {
            let cmix_dm_SendText_return { r0, r1 } = cmix_dm_SendText(
                self.instance_id,
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
        &self,
        partner_pubkey: &[u8],
        dm_token: i32,
        message: &str,
        reply_to: &[u8],
        lease_time_ms: i64,
        cmix_params_json: &[u8],
    ) -> Result<Vec<u8>, String> {
        unsafe {
            let cmix_dm_SendReply_return { r0, r1 } = cmix_dm_SendReply(
                self.instance_id,
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
        &self,
        partner_pubkey: &[u8],
        dm_token: i32,
        message: &str,
        react_to: &[u8],
        lease_time_ms: i64,
        cmix_params_json: &[u8],
    ) -> Result<Vec<u8>, String> {
        unsafe {
            let cmix_dm_SendReaction_return { r0, r1 } = cmix_dm_SendReaction(
                self.instance_id,
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
