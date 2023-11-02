use std::time::{Duration, Instant};

use crate::{
    net_util::{PacketType, WriteSocket},
    rodio_util::Output,
};

pub struct RingManager {
    last_ring_sound: Option<Instant>,
    last_ring_packet: Option<Instant>,
}

impl RingManager {
    pub fn new() -> RingManager {
        return RingManager {
            last_ring_sound: None,
            last_ring_packet: None,
        };
    }

    pub fn play_sound(&mut self, output: &Output) {
        run_with_debounce(&mut self.last_ring_sound, Duration::from_secs(4), || {
            output.play_ring(2.0, 0.05, Duration::from_secs(2));
        });
    }

    pub fn send_packet(&mut self, write_socket: &WriteSocket, packet_type: PacketType) {
        run_with_debounce(
            &mut self.last_ring_packet,
            Duration::from_millis(100),
            || {
                write_socket.send(packet_type);
            },
        );
    }

    pub fn reset(&mut self) {
        self.last_ring_sound = None;
        self.last_ring_packet = None;
    }
}

fn run_with_debounce<T: FnOnce()>(prev_time: &mut Option<Instant>, delta: Duration, action: T) {
    let now = Instant::now();
    match prev_time {
        Some(x) if now.duration_since(*x) > delta => {
            *prev_time = Some(now);
            action()
        }
        None => {
            *prev_time = Some(now);
            action()
        }
        _ => {}
    }
}
