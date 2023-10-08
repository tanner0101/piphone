use rodio::cpal;
use rodio::DeviceTrait;
use rodio::Source;
use std;
use std::thread;
use std::time;

pub fn find_device_by_name(
    devices: std::iter::Filter<rodio::Devices, fn(&<rodio::Devices as Iterator>::Item) -> bool>,
    target_name: &str,
) -> Option<cpal::Device> {
    for device in devices {
        let name = device.name().unwrap();
        println!("found i/o device {}", name);
        if name == target_name {
            println!("    -> chose {}", name);
            return Some(device);
        }
    }

    println!("ERROR: Unble to find device {}", target_name);

    return None;
}

// plays a ringing noise n times
#[allow(dead_code)]
pub fn play_ring(n: usize, sink: &rodio::Sink) {
    for _ in 0..n {
        let hi = rodio::source::SineWave::new(420.0)
            .amplify(1.0)
            .take_duration(time::Duration::from_secs(2));
        let lo = rodio::source::SineWave::new(69.0)
            .amplify(1.0)
            .take_duration(time::Duration::from_secs(2));
        let ring_tone = hi.mix(lo);
        sink.append(ring_tone);
        sink.sleep_until_end();
        thread::sleep(time::Duration::from_secs(4));
    }
}

// plays a quiet noise to prepare the speaker
#[allow(dead_code)]
pub fn ready_speaker(sink: &rodio::Sink) {
    let init_tone = rodio::source::SineWave::new(440.0)
        .take_duration(time::Duration::from_secs(4))
        .amplify(0.01); // very low volume
    sink.append(init_tone);
    sink.sleep_until_end();
}
