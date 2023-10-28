use std::net::UdpSocket;

pub struct WriteSocket {
    socket: UdpSocket,
    remote_addr: String,
}

impl WriteSocket {
    pub fn new(peer_addr: String, port: u32) -> WriteSocket {
        let remote_addr = format!("{}:{}", peer_addr, port);
        return WriteSocket::_new(remote_addr);
    }

    fn _new(remote_addr: String) -> WriteSocket {
        // bind to random local port for sending.
        // remote_addr is passed in socket send_to.
        let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind send socket");

        return WriteSocket {
            socket,
            remote_addr,
        };
    }

    // Initialize a new Connection object based on an existing one
    pub fn duplicate(&self) -> WriteSocket {
        return WriteSocket::_new(self.remote_addr.clone());
    }

    pub fn send(&self, packet_type: PacketType) {
        self.send_data(packet_type, &[]);
    }

    pub fn send_data(&self, packet_type: PacketType, data: &[u8]) {
        let header: [u8; 1] = [packet_type.to_raw(); 1];
        let packet = [&header, data].concat();
        self.socket
            .send_to(&packet, &self.remote_addr)
            .expect("failed to send packet");
    }
}

pub struct ReadSocket {
    socket: UdpSocket,
    buf: [u8; 32768],
}

impl ReadSocket {
    pub fn new(port: u32) -> ReadSocket {
        let buf: [u8; 32768] = [0u8; 16_384 * 2];
        let socket =
            UdpSocket::bind(format!("0.0.0.0:{}", port)).expect("Failed to bind recv socket");
        socket
            .set_nonblocking(true)
            .expect("set_nonblocking call failed");
        return ReadSocket { socket, buf };
    }

    pub fn read(&mut self) -> Option<Packet> {
        return match self.socket.recv_from(&mut self.buf) {
            Ok((len, _src)) => Some(Packet {
                packet_type: PacketType::from_raw(self.buf[0]),
                data: &self.buf[1..len],
            }),
            Err(_) => None,
        };
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Packet<'a> {
    pub packet_type: PacketType,
    pub data: &'a [u8],
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PacketType {
    VoiceData,
    Ring,
    RingAck,
}

impl PacketType {
    pub fn from_raw(value: u8) -> PacketType {
        return match value {
            1 => PacketType::VoiceData,
            2 => PacketType::Ring,
            3 => PacketType::RingAck,
            _ => panic!("Invalid packet type {}", value),
        };
    }

    pub fn to_raw(self) -> u8 {
        return match self {
            PacketType::VoiceData => 1,
            PacketType::Ring => 2,
            PacketType::RingAck => 3,
        };
    }
}
