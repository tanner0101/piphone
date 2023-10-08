pub struct Config {
    pub peer_addr: String,
    pub port: i32,
    pub input_device_name: String,
    pub output_device_name: String,
}

impl Config {
    pub fn new() -> Self {
        return Config {
            peer_addr: if Config::is_tanner() { 
                String::from("72.180.246.104")
            } else {
                String::from("72.180.248.254")
            },
            port: 5060,
            input_device_name: String::from("sysdefault:CARD=Device"),
            output_device_name: String::from("sysdefault:CARD=Device"),
        }
    }

    pub fn is_tanner() -> bool {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let path_str = current_dir.display().to_string();

        return path_str.contains("tanner");
    }
}
