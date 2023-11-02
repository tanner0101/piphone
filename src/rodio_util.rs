use rodio::cpal::traits::{HostTrait, StreamTrait};
use rodio::{cpal, DeviceTrait, Source};
use std::{self, thread, time, process};

pub struct Input {
    mic_cfg: rodio::SupportedStreamConfig,
    _mic_stream: cpal::Stream,
}

impl Input {
    pub fn open<T: FnMut(&[u8]) + Send + 'static>(
        device_name: &str,
        mut data_callback: T,
    ) -> Input {
        let device =
            find_device_by_name(cpal::default_host().input_devices().unwrap(), &device_name)
                .unwrap();

        let mic_cfg = device.default_input_config().unwrap();
        let _mic_stream = device
            .build_input_stream(
                &mic_cfg.clone().into(),
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    data_callback(Input::to_network(&data));
                },
                move |err| {
                    eprintln!("an error occurred on stream: {}", err);
                    // Exit process so that systemd can restart us
                    process::exit(1);
                },
                Option::None,
            )
            .unwrap();

        _mic_stream.play().unwrap();

        return Input {
            mic_cfg,
            _mic_stream,
        };
    }

    fn to_network(data: &[i16]) -> &[u8] {
        let len = data.len() * std::mem::size_of::<i16>();
        let ptr = data.as_ptr() as *const u8;
        return unsafe { std::slice::from_raw_parts(ptr, len) };
    }

    fn from_network(data: &[u8]) -> &[i16] {
        let ptr = data.as_ptr() as *const i16;
        let pcm_len = data.len() / std::mem::size_of::<i16>();
        return unsafe { std::slice::from_raw_parts(ptr, pcm_len) };
    }
}

pub struct Output {
    sink: rodio::Sink,
    _os: rodio::OutputStream,
    _device: cpal::Device,
}

impl Output {
    pub fn open(device_name: &str) -> Output {
        let device =
            find_device_by_name(cpal::default_host().output_devices().unwrap(), &device_name)
                .unwrap();
        let (_os, output_stream_handle) = rodio::OutputStream::try_from_device(&device).unwrap();
        let sink = rodio::Sink::try_new(&output_stream_handle).unwrap();
        return Output {
            sink: sink,
            _os: _os,
            _device: device,
        };
    }

    // plays a 4 second ringing noise
    pub fn play_ring(&self, freq: f32, amp: f32, time: time::Duration) {
        let hi = rodio::source::SineWave::new(420.0 * freq)
            .amplify(amp)
            .take_duration(time);
        let lo = rodio::source::SineWave::new(69.0 * freq)
            .amplify(amp)
            .take_duration(time);
        let ring_tone = hi.mix(lo);
        self.sink.append(ring_tone);
    }

    // play a ring sound to indicate that the device is ready
    pub fn play_start_tone(&self) {
        self.ready_speaker();
        let init_tone_dur = time::Duration::from_millis(100);
        self.play_ring(3.0, 1.0, init_tone_dur);
        thread::sleep(init_tone_dur);
        self.play_ring(4.0, 1.0, init_tone_dur);
        thread::sleep(init_tone_dur);
        self.play_ring(5.0, 1.0, init_tone_dur);
        thread::sleep(init_tone_dur);
    }

    // plays a quiet noise to prepare the speaker
    pub fn ready_speaker(&self) {
        let init_tone = rodio::source::SineWave::new(440.0)
            .take_duration(time::Duration::from_millis(100))
            .amplify(0.01); // very low volume
        self.sink.append(init_tone);
        self.sink.sleep_until_end();
    }

    // source input is needed to match configs
    pub fn play_data(&self, data: &[u8], source_input: &Input) {
        let source = rodio::buffer::SamplesBuffer::new(
            source_input.mic_cfg.channels(),
            source_input.mic_cfg.sample_rate().0,
            Input::from_network(data),
        );
        self.sink.append(source);
    }

    pub fn stop(&self) {
        self.sink.stop();
    }
}

fn find_device_by_name(
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

    return None;
}
