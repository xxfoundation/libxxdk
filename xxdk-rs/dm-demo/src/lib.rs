use std::path::PathBuf;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use base64::prelude::*;
use structopt::StructOpt;
use xxdk::base::callbacks::DmCallbacks;
use xxdk::base::*;

const SECRET: &str = "Hello";
const REGISTRATION_CODE: &str = "";
const DM_ID_EKV_KEY: &str = "MyDMID";

#[derive(Debug, structopt::StructOpt)]
pub struct Options {
    /// Path to network definition file
    #[structopt(long)]
    pub ndf: PathBuf,

    /// Path to state directory
    #[structopt(long)]
    pub state_dir: String,

    /// Partner's public key
    ///
    /// If omitted, send messages to self.
    #[structopt(long)]
    pub partner_key: Option<String>,

    /// Partner's chat token
    ///
    /// If omitted or 0, send messages to self.
    #[structopt(long, default_value = "0")]
    pub partner_token: i32,

    /// Message to send
    #[structopt(long, default_value = "Hello, world!")]
    pub message: String,

    /// Seconds to wait for messages
    #[structopt(long, default_value = "20")]
    pub wait: u32,

    /// Number of messages to wait for
    #[structopt(long, default_value = "1")]
    pub receive_count: i64,
}

pub fn run() -> Result<(), String> {
    let options = Options::from_args();

    let ndf_contents = std::fs::read_to_string(&options.ndf).map_err(|e| e.to_string())?;

    println!("[Demo] ======== Rust xxdk DM demo =========");
    println!(
        "[Demo] xxdk-client version: {}\n",
        xxdk::base::get_version()
    );

    if std::fs::read_dir(&options.state_dir).is_err() {
        CMix::create(
            &ndf_contents,
            &options.state_dir,
            SECRET.as_bytes(),
            REGISTRATION_CODE,
        )?;
    }

    let cmix = CMix::load(&options.state_dir, SECRET.as_bytes(), &[])?;
    let reception_id = cmix.reception_id()?;
    println!(
        "[Demo] cMix reception ID: {}",
        BASE64_STANDARD.encode(reception_id)
    );

    let dm_id = cmix.ekv_get(DM_ID_EKV_KEY).or_else(|_| {
        println!("[Demo] Generating DM identity...");

        let id = generate_codename_identity(SECRET);
        cmix.ekv_set(DM_ID_EKV_KEY, &id)?;

        println!(
            "[Demo] Exported codename blob: {}",
            BASE64_STANDARD.encode(&id)
        );

        Ok::<_, String>(id)
    })?;

    let cbs = Arc::new(DemoCallbacks::new());
    let dm = cmix.new_dm_client(&dm_id, SECRET, Arc::clone(&cbs) as Arc<dyn DmCallbacks>)?;

    let my_token = dm.get_token()?;
    let my_pubkey = dm.get_dm_pubkey()?;

    println!("[Demo] DM Pubkey: {}", BASE64_STANDARD.encode(&my_pubkey));
    println!("[Demo] DM Token: {my_token}");

    let (partner_token, partner_pubkey) = match (&options.partner_key, options.partner_token) {
        (Some(key_b64), tok) if tok != 0 => {
            let key = BASE64_STANDARD.decode(key_b64).map_err(|e| e.to_string())?;
            (tok, key)
        }

        _ => {
            println!("[Demo] Partner key or token missing, sending to self");
            (my_token, my_pubkey.clone())
        }
    };

    println!(
        "[Demo] Partner DM Pubkey: {}",
        BASE64_STANDARD.encode(&partner_pubkey)
    );
    println!("[Demo] Partner DM Token: {partner_token}");

    cmix.start_network_follower(5000)?;

    while let Err(e) = cmix.wait_for_network(20000) {
        println!("[Demo] Waiting to connect to network: {e}");
    }

    while !cmix.ready_to_send() {
        std::thread::sleep(Duration::from_secs(1));
    }

    dm.send_text(&partner_pubkey, my_token, &options.message, 0, &[])?;

    let mut times_waited = 0;
    while cbs.num_received() < options.receive_count && times_waited < options.wait {
        println!("[Demo] Num received: {}", cbs.num_received());
        std::thread::sleep(Duration::from_secs(1));
        times_waited += 1;
    }

    if times_waited >= options.wait {
        println!("[Demo] Timed out waiting for messages");
    }

    println!("[Demo] Num received: {}. Exiting", cbs.num_received());

    cmix.stop_network_follower()
}

pub struct DemoCallbacks {
    pub num_received: Arc<AtomicI64>,
}

impl DemoCallbacks {
    pub fn new() -> Self {
        Self {
            num_received: Arc::new(AtomicI64::new(0)),
        }
    }

    pub fn num_received(&self) -> i64 {
        self.num_received.load(Ordering::Relaxed)
    }

    pub fn increment_num_received(&self) {
        self.num_received.fetch_add(1, Ordering::Relaxed);
    }
}

impl Default for DemoCallbacks {
    fn default() -> Self {
        Self::new()
    }
}

impl DmCallbacks for DemoCallbacks {
    fn receive(
        &self,
        _message_id: &[u8],
        _nickname: &str,
        text: &[u8],
        _partner_key: &[u8],
        _sender_key: &[u8],
        _dm_token: i32,
        _codeset: i32,
        _timestamp: i64,
        _round_id: i64,
        _message_type: i64,
        _status: i64,
    ) -> i64 {
        println!(
            "[Demo] DMReceiveCallbackFn: {}",
            BASE64_STANDARD.encode(text)
        );
        self.increment_num_received();
        self.num_received()
    }

    fn receive_text(
        &self,
        _message_id: &[u8],
        _nickname: &str,
        text: &str,
        _partner_key: &[u8],
        _sender_key: &[u8],
        _dm_token: i32,
        _codeset: i32,
        _timestamp: i64,
        _round_id: i64,
        _status: i64,
    ) -> i64 {
        println!("[Demo] DMReceiveTextCallbackFn: {text}");
        self.increment_num_received();
        self.num_received()
    }

    fn receive_reply(
        &self,
        _message_id: &[u8],
        _reply_to: &[u8],
        _nickname: &str,
        text: &str,
        _partner_key: &[u8],
        _sender_key: &[u8],
        _dm_token: i32,
        _codeset: i32,
        _timestamp: i64,
        _round_id: i64,
        _status: i64,
    ) -> i64 {
        println!("[Demo] DMReceiveReplyCallbackFn: {text}");
        self.increment_num_received();
        self.num_received()
    }

    fn receive_reaction(
        &self,
        _message_id: &[u8],
        _reaction_to: &[u8],
        _nickname: &str,
        text: &str,
        _partner_key: &[u8],
        _sender_key: &[u8],
        _dm_token: i32,
        _codeset: i32,
        _timestamp: i64,
        _round_id: i64,
        _status: i64,
    ) -> i64 {
        println!("[Demo] DMReceiveReactionCallbackFn: {text}");
        self.increment_num_received();
        self.num_received()
    }

    fn update_sent_status(
        &self,
        _uuid: i64,
        _message_id: &[u8],
        _timestamp: i64,
        _round_id: i64,
        _status: i64,
    ) {
        println!("[Demo] DMUpdateSentStatusCallbackFn");
        self.increment_num_received();
    }

    fn block_sender(&self, _pubkey: &[u8]) {}

    fn unblock_sender(&self, _pubkey: &[u8]) {}

    fn get_conversation(&self, _pubkey: &[u8]) -> Vec<u8> {
        Vec::new()
    }

    fn get_conversations(&self) -> Vec<u8> {
        Vec::new()
    }

    fn delete_message(&self, _message_id: &[u8], _pubkey: &[u8]) -> bool {
        false
    }

    fn event_update(&self, _event_type: i64, json_data: &[u8]) {
        println!(
            "[Demo] Event update: {}",
            String::from_utf8_lossy(json_data)
        );
    }
}
