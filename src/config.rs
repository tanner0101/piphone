pub struct Config {
    pub peer_addr: String,
    pub port: i32,
    pub input_device_name: String,
    pub output_device_name: String,
}

impl Config {
    pub fn new() -> Self {
        return Config {
            peer_addr: String::from("72.180.248.254"),
            port: 5060,
            input_device_name: String::from("sysdefault:CARD=Device"),
            output_device_name: String::from("sysdefault:CARD=Device"),
        }
    }
}
