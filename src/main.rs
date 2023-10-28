mod call_state_machine;
mod config;
mod gpio_util;
mod net_util;
mod ring;
mod rodio_util;

use std::sync::{Arc, RwLock};
use std::{thread, time};

use crate::call_state_machine::*;
use crate::config::Config;
use crate::net_util::{PacketType, ReadSocket, WriteSocket};
use crate::ring::RingManager;
use crate::rodio_util::{Input, Output};

fn main() {
    let cfg = Config::new();
    let mut read_socket = ReadSocket::new(cfg.port);
    let write_socket = WriteSocket::new(cfg.peer_addr.clone(), cfg.port);
    println!("connecting to {}:{}...", cfg.peer_addr, cfg.port);

    // setup state variables
    let call = Arc::new(RwLock::new(Call::new()));
    let mut call_switch = CallSwitch::new();
    let mut ringer = RingManager::new();

    println!("\nstarting microphone...");

    // create state and socket copies for microphone thread
    let call_copy = Arc::clone(&call);
    let write_socket_copy = write_socket.duplicate();

    let microphone = Input::open(&cfg.headset_in_device, move |data: &[u8]| {
        let call_state = call_copy
            .read()
            .expect("failed to get call lock")
            .state
            .clone();
        match call_state {
            CallState::InProgressCall => {
                write_socket_copy.send_data(PacketType::VoiceData, data);
            }
            _ => {}
        }
    });

    println!("\nstarting speakers...");

    let headset = Output::open(&cfg.headset_in_device);
    let speaker = Output::open(&cfg.ring_out_device);

    speaker.play_start_tone();

    let mut previous_call_state: Option<CallState> = None;
    loop {
        // read packet (if any) and check call active gpio.
        // then dispatch state machines
        let packet = read_socket.read();
        let packet_type = match packet {
            Some(packet) => Some(packet.packet_type),
            None => None,
        };
        let call_switch_edge = call_switch.dispatch(&match gpio_util::read_pin("6") {
            1 => CallSwitchState::Active,
            _ => CallSwitchState::Inactive,
        });
        let call_state = call
            .write()
            .expect("failed to get call lock")
            .dispatch(&packet_type, &call_switch_edge)
            .state
            .clone();

        use CallState::*;

        // stop any in-progress sounds on hang up or answer
        match (previous_call_state, call_state) {
            (Some(IncomingCall), Idle | InProgressCall) => speaker.stop(),
            (Some(OutgoingCall), Idle | InProgressCall) => headset.stop(),
            _ => {}
        }

        // reset ringer debounce if we've changed state
        if previous_call_state != Some(call_state) {
            ringer.reset();
        }

        // play sounds and send packets based on call state
        match call_state {
            CallState::InProgressCall => match packet {
                Some(packet) => {
                    headset.play_data(packet.data, &microphone);
                }
                None => {}
            },
            CallState::IncomingCall => {
                ringer.play_sound(&speaker);
                ringer.send_packet(&write_socket, PacketType::RingAck);
            }
            CallState::OutgoingCall => {
                ringer.play_sound(&headset);
                ringer.send_packet(&write_socket, PacketType::Ring);
            }
            CallState::Idle => {
                // avoid cpu hogging when we're not doing anything
                thread::sleep(time::Duration::from_millis(100));
            }
        }

        previous_call_state = Some(call_state);
    }
}
