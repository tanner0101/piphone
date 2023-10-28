#[derive(Clone)]
pub struct Config {
    pub peer_addr: String,
    pub port: u32,
    pub headset_in_device: String,
    pub headset_out_device: String,
    pub ring_out_device: String,
}

const PORT: u32 = 5060;
const TANNER_IP: &str = "72.180.248.254";
const JACOB_IP: &str = "tanneristheworst.asuscomm.com"; // 72.180.246.104

impl Config {
    pub fn new() -> Self {
        if Config::is_tanner() {
            return Config {
                peer_addr: JACOB_IP.to_string(),
                port: PORT,
                headset_in_device: String::from("sysdefault:CARD=Device"),
                headset_out_device: String::from("sysdefault:CARD=Device"),
                ring_out_device: String::from("sysdefault:CARD=rockchiprk809"),
            };
        }

        // Implicitly, is_jacob()
        return Config {
            peer_addr: TANNER_IP.to_string(),
            port: PORT,
            headset_in_device: String::from("sysdefault:CARD=MV7"),
            headset_out_device: String::from("sysdefault:CARD=MV7"),
            ring_out_device: String::from("sysdefault:CARD="),
        };
    }

    pub fn is_tanner() -> bool {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let path_str = current_dir.display().to_string();

        return path_str.contains("tanner");
    }
}
