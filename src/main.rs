use std;
use std::net;
use std::io;
use std::process;
use std::thread;
use std::time;
use biquad::Biquad;
use biquad::ToHertz;
use rodio::DeviceTrait;
use rodio::Source;
use rodio::cpal;
use rodio::cpal::traits::HostTrait;
use rodio::cpal::traits::StreamTrait;

fn main() -> io::Result<()> {
    let ctx = Context::new("72.180.248.254");
    let (_device, mic_config, _stream) = start_microphone(&ctx);
    start_speaker(&ctx, mic_config);
    return io::Result::Ok(());
}

struct Context {
    remote_addr: String,
    send_socket: net::UdpSocket,
    recv_socket: net::UdpSocket,
}

impl Context {
    fn new(remote_addr: &str) -> Self {
        return Context {
            remote_addr: format!("{}:5060", remote_addr),
            send_socket: net::UdpSocket::bind("0.0.0.0:0")
                .expect("Failed to bind send socket"),
            recv_socket: net::UdpSocket::bind("0.0.0.0:5060")
                .expect("Failed to bind recv socket"),
        }
    }
}

fn start_microphone(ctx: &Context) -> (rodio::Device, rodio::SupportedStreamConfig, cpal::Stream) {
    println!("\nstarting microphone...");
    let host = cpal::default_host();
    let device = find_device_by_name(
        host.input_devices().unwrap(),
        "sysdefault:CARD=X",
    ).unwrap();
    let config = device.default_input_config().unwrap();
    let mut filter = PoormansFilter::new(config.sample_rate().0.hz());
    let send_socket = ctx.send_socket.try_clone().unwrap();
    let remote_addr = ctx.remote_addr.clone();
    let input_stream = device.build_input_stream(
        &config.clone().into(),
        move |_data: &[i16], _: &cpal::InputCallbackInfo| {
            if !is_call_active() {
                return;
            }
            let len = _data.len() * std::mem::size_of::<i16>();
            let data: Vec<i16> = _data.iter().map(|x| filter.run(*x)).collect();
            let ptr = data.as_ptr() as *const u8;
            let slice_u8: &[u8] = unsafe { std::slice::from_raw_parts(ptr, len) };
            send_socket.send_to(slice_u8, &remote_addr).expect("failed to send mic data");
        },
        move |err| {
            eprintln!("an error occurred on stream: {}", err);
        },
        Option::None,
    )
        .unwrap();

    input_stream
        .play()
        .unwrap();

    return (device, config, input_stream)
}

fn start_speaker(ctx: &Context, config: rodio::SupportedStreamConfig) {
    println!("\nstarting speaker...");
    let device = find_device_by_name(
        cpal::default_host().output_devices().unwrap(),
        "sysdefault:CARD=X",
    ).unwrap();
    let (_os, output_stream_handle) = rodio::OutputStream::try_from_device(&device).unwrap();
    let sink = rodio::Sink::try_new(&output_stream_handle).unwrap();
    let mut buf = [0u8; 16_384*2];
    println!("starting loop");
    loop {
        let (amt, _src) = ctx.recv_socket.recv_from(&mut buf)
            .expect("Failed to read socket");
        if !is_call_active() {
            continue;
        }

        let ptr = buf.as_ptr() as *const i16;
        let len = amt / std::mem::size_of::<i16>();
        let samples: &[i16] = unsafe { std::slice::from_raw_parts(ptr, len) };

        let source = rodio::buffer::SamplesBuffer::new(
            config.channels(),
            config.sample_rate().0,
            samples.clone(),
        );
        sink.append(source);
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

    return None
}

fn is_call_active() -> bool {
    return read_gpio("6") == 1;
}

fn read_gpio(id: &str) -> u8 {
    let mut cmd = process::Command::new("gpio");
    cmd.arg("read").arg(id);
    let hi = String::from_utf8_lossy(&cmd.output().unwrap().stdout).trim() == "1"; 
    if hi {
        return 1;
    } else {
        return 0;
    }
}

struct PoormansFilter {
    lp: biquad::DirectForm1<f32>,
    hp: biquad::DirectForm1<f32>,
}

impl PoormansFilter {
    fn new(sample_rate: biquad::Hertz<f32>) -> Self {
        let cutoff_frequency = 3.khz();
        let lp_coeffs = biquad::Coefficients::<f32>::from_params(
            biquad::Type::LowPass, 
            sample_rate, 
            cutoff_frequency, 
            biquad::Q_BUTTERWORTH_F32).unwrap();

        let hp_coeffs = biquad::Coefficients::<f32>::from_params(
            biquad::Type::HighPass, 
            sample_rate, 
            120.hz(), 
            biquad::Q_BUTTERWORTH_F32).unwrap();

        return PoormansFilter {
            lp: biquad::DirectForm1::<f32>::new(lp_coeffs),
            hp: biquad::DirectForm1::<f32>::new(hp_coeffs),
        }
    }

    fn run(&mut self, x: i16) -> i16 {
        return self.hp.run(self.lp.run(x as f32)).round() as i16;
    }
}

// plays a ringing noise n times
#[allow(dead_code)]
fn play_ring(n: usize, sink: &rodio::Sink) {
    for _ in 0..n {
        let hi = rodio::source::SineWave::new(420.0).amplify(1.0).take_duration(time::Duration::from_secs(2));
        let lo = rodio::source::SineWave::new(69.0).amplify(1.0).take_duration(time::Duration::from_secs(2));
        let ring_tone = hi.mix(lo) ;
        sink.append(ring_tone);
        sink.sleep_until_end();
        thread::sleep(time::Duration::from_secs(4));
    }
}

// plays a quiet noise to prepare the speaker
#[allow(dead_code)]
fn ready_speaker(sink: &rodio::Sink) {
    let init_tone = rodio::source::SineWave::new(440.0)
        .take_duration(time::Duration::from_secs(4))
        .amplify(0.01); // very low volume
    sink.append(init_tone);
    sink.sleep_until_end();
}
