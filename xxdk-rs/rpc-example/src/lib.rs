use std::path::PathBuf;
use std::sync::Arc;

use base64::prelude::*;
use structopt::StructOpt;
use xxdk::base::*;
use xxdk::service::*;

const SECRET: &str = "Hello";
const REGISTRATION_CODE: &str = "";

#[derive(Debug, structopt::StructOpt)]
pub struct Options {
    /// Path to network definition file
    #[structopt(long)]
    pub ndf: PathBuf,

    /// Path to state directory
    #[structopt(long)]
    pub state_dir: String,
}

pub fn run() -> Result<(), String> {
    let options = Options::from_args();

    let ndf_contents = std::fs::read_to_string(&options.ndf).map_err(|e| e.to_string())?;

    println!("[Demo] ======== Rust xxdk RPC demo =========");
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
        BASE64_STANDARD.encode(&reception_id)
    );

    let cmix_config = CMixServerConfig {
        ndf_path: String::from(options.ndf.to_str().unwrap()),
        storage_dir: options.state_dir,
        secret: String::from(SECRET),
        reception_id: BASE64_STANDARD_NO_PAD.encode(&reception_id),
        private_key: String::from(""),
    };

    let xx_router = xxdk::service::Router::new(xx_rpc_handler, Arc::new(cmix));
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(6) // Set the number of Tokio worker threads
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(CMixServer::serve(xx_router, cmix_config))
}

pub async fn xx_rpc_handler(_: Arc<CMix>, request: IncomingRequest) -> Result<Vec<u8>, String> {
    let sender: String = request
        .sender_id
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    tracing::info!(sender, "Received message via cMix",);
    let text = String::from_utf8_lossy(&request.request);

    let mut resp = Vec::from(b"Hi from rust rpc example! Echoed message: {\"root\":\"0x2dba44d695326db3af5a8a25caf39330adeedfdd6a4738e30927f98bb1e4d9d9\",\"proof\":[{\"Left\":\"0x2793fc16cbd7b3340525b40bb46c1d7e4536ea6e1af9d96e13edd06052aaba63\"},{\"Right\":\"0x1f564825ad6f7c68443c9bc096295602c17339c2557e441b38dad0720ec5e9f1\"},{\"Left\":\"0x217c4c12af3b1b49c877960a5928899b060c2950a24bc92bb5a14af4524eb38\"},{\"Left\":\"0x2f309bf1cdd91b784dc4cbaec44881809750cac06ce2f4d3a411592c22ce7e40\"},{\"Left\":\"0x56d4471d170ba29faeacd33eb709150ae25daefe02bc8a515c84d389670cfd9\"},{\"Left\":\"0x22443ea99c74905cea7f0a4e7f537aec9c59d398e841d8d48a09859646ca965\"},{\"Left\":\"0xe4b4c5cdf28b5600fc26effa762c2996b3d6b8a92928803a95864a9fc2fd9e4\"},{\"Left\":\"0x2264260d21a5760b5dceec38507fce1c1ae6aa57ef352df147ae915b6eb2231f\"},{\"Left\":\"0x14f486de9e25f3e3e33b035d132b73cfd4d75e3285a6172d5534f85fd55a995a\"},{\"Left\":\"0x3219565149732ed57b90233f40df7e695127d5fb72ca5285f0b36166b2b9ce1\"},{\"Left\":\"0x19c489fbd3f07145fbbffb8fc8dd11e4e80f17963eb2f01aa7f76e3043137ba5\"},{\"Left\":\"0x12e5fbf330fe9950dc1f5fde62642e45cff737c569c13d07a8a93239a35c6a20\"},{\"Left\":\"0xc9bbaf5e960cfecf523868bcd0db2113df69e52d7f6c6f531c8358dcb984743\"},{\"Left\":\"0x16c7fb6a8aa40724b813d958dce15a8c3685354868267b6472eb24627754f915\"},{\"Left\":\"0x48c63ec390c6e641f1df2f8ab0063bc1e2c45a7682467ee260c52c81cc8ac15\"},{\"Left\":\"0xa5194a89cdd0f276e500e628ea7b10036dde1b89ef78be7421b8445f93a3c20\"},{\"Left\":\"0x2a7c7c9b6ce5880b9f6f228d72bf6a575a526f29c66ecceef8b753d38bba7323\"},{\"Left\":\"0x2e8186e558698ec1c67af9c14d463ffc470043c9c2988b954d75dd643f36b992\"},{\"Left\":\"0xf57c5571e9a4eab49e2c8cf050dae948aef6ead647392273546249d1c1ff10f\"},{\"Left\":\"0x1830ee67b5fb554ad5f63d4388800e1cfe78e310697d46e43c9ce36134f72cca\"},{\"Left\":\"0x2134e76ac5d21aab186c2be1dd8f84ee880a1e46eaf712f9d371b6df22191f3e\"},{\"Left\":\"0x19df90ec844ebc4ffeebd866f33859b0c051d8c958ee3aa88f8f8df3db91a5b1\"},{\"Left\":\"0x18cca2a66b5c0787981e69aefd84852d74af0e93ef4912b4648c05f722efe52b\"},{\"Left\":\"0x2388909415230d1b4d1304d2d54f473a628338f2efad83fadf05644549d2538d\"},{\"Left\":\"0x27171fb4a97b6cc0e9e8f543b5294de866a2af2c9c8d0b1d96e673e4529ed540\"},{\"Left\":\"0x2ff6650540f629fd5711a0bc74fc0d28dcb230b9392583e5f8d59696dde6ae21\"},{\"Left\":\"0x120c58f143d491e95902f7f5277778a2e0ad5168f6add75669932630ce611518\"},{\"Left\":\"0x1f21feb70d3f21b07bf853d5e5db03071ec495a0a565a21da2d665d279483795\"},{\"Left\":\"0x24be905fa71335e14c638cc0f66a8623a826e768068a9e968bb1a1dde18a72d2\"},{\"Left\":\"0xf8666b62ed17491c50ceadead57d4cd597ef3821d65c328744c74e553dac26d\"}]}");
    resp.extend_from_slice(text.as_bytes());
    Ok(resp)
}
