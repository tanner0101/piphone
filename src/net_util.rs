use std;
use std::net;

pub struct Context {
    pub remote_addr: String,
    pub send_socket: net::UdpSocket,
    pub recv_socket: net::UdpSocket,
}

impl Context {
    pub fn new(remote_addr: &str) -> Self {
        return Context {
            remote_addr: format!("{}:5060", remote_addr),
            send_socket: net::UdpSocket::bind("0.0.0.0:0")
                .expect("Failed to bind send socket"),
            recv_socket: net::UdpSocket::bind("0.0.0.0:5060")
                .expect("Failed to bind recv socket"),
        }
    }
}
