use crate::net::omni::OmniPeerId;
use crate::AudioSettings;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use opus::{Application, Channels, Decoder, Encoder};
use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use rubato::{FftFixedIn, Resampler};
use std::collections::HashMap;
use std::ops::Mul;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;
use tracing::error;
use tracing::log::warn;

pub const SAMPLE_RATE: usize = 24000;
pub const FRAME_SIZE: usize = 480;
pub const CHANNELS: Channels = Channels::Mono;

/*/// For reference, Mina is 14 pixels high.
const PIXELS_PER_METER: f32 = 14.0 / 1.7;
/// In m/s.
const SPEED_OF_SOUND: f32 = 343.0;

const SAMPLE_COUNT: usize = 4;

struct VelocityTracker {
    prev_pos_push: Instant,
    source_pos: [WorldPos; SAMPLE_COUNT],
    player_pos: [WorldPos; SAMPLE_COUNT],
}

impl Default for VelocityTracker {
    fn default() -> Self {
        Self {
            prev_pos_push: Instant::now(),
            source_pos: Default::default(),
            player_pos: Default::default(),
        }
    }
}

impl VelocityTracker {
    fn push_new_pos(&mut self, source: WorldPos, player: WorldPos) {
        if self.prev_pos_push.elapsed() > Duration::from_millis(16 * 4) {
            self.source_pos.rotate_right(1);
            self.source_pos[0] = source;
            self.player_pos.rotate_right(1);
            self.player_pos[0] = player;
            self.prev_pos_push = Instant::now();
        }
    }
    fn calc_speed_toward_player(&self) -> f32 {
        let vel_source = self.source_pos[0] - self.source_pos[SAMPLE_COUNT - 1];
        let vel_player = self.player_pos[0] - self.player_pos[SAMPLE_COUNT - 1];
        let vel = vel_source - vel_player;
        let source_to_player = self.player_pos[0] - self.source_pos[0];
        let ret = vel.dot(source_to_player) / source_to_player.hypot() / PIXELS_PER_METER;
        if ret.is_nan() {
            0.0
        } else {
            ret
        }
    }
}

const DSP_PITCH_MULT: usize = 100;

fn make_dsp() -> (Arc<AtomicUsize>, BigBlockAdapter) {
    let pitch_control = Arc::new(AtomicUsize::new(DSP_PITCH_MULT));

    let node = resynth::<U1, U1, _>(1 << 13, {
        let pitch_control = pitch_control.clone();
        move |fft| {
            let p = pitch_control.load(std::sync::atomic::Ordering::Relaxed);
            for i in 0..fft.bins() {
                let j = i * p / DSP_PITCH_MULT;
                if (0..fft.bins()).contains(&j) {
                    fft.set(0, i, fft.at(0, j));
                }
            }
        }
    });
    (pitch_control, BigBlockAdapter::new(Box::new(node)))
}*/
struct PlayerInfo {
    //tracker: VelocityTracker,
    sink: Sink,
    //dsp: BigBlockAdapter,
    //pitch_control: Arc<AtomicUsize>,
}

pub(crate) struct AudioManager {
    per_player: HashMap<OmniPeerId, PlayerInfo>,
    stream_handle: Option<(OutputStream, OutputStreamHandle)>,
    decoder: Decoder,
    rx: Receiver<Vec<u8>>,
}

impl AudioManager {
    pub fn new(audio: AudioSettings) -> Self {
        #[cfg(target_os = "linux")]
        let host = cpal::available_hosts()
            .into_iter()
            .find(|id| *id == cpal::HostId::Jack)
            .and_then(|id| cpal::host_from_id(id).ok())
            .unwrap_or(cpal::default_host());
        #[cfg(not(target_os = "linux"))]
        let host = cpal::default_host();

        let device = {
            let input = audio.input_device.clone();
            if audio.disabled {
                None
            } else if input.is_none() {
                host.default_input_device()
            } else if let Some(d) = host
                .input_devices()
                .map(|mut d| d.find(|d| d.name().ok() == input))
                .ok()
                .flatten()
            {
                Some(d)
            } else {
                host.default_input_device()
            }
        };
        let decoder = Decoder::new(SAMPLE_RATE as u32, CHANNELS).unwrap();
        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        thread::spawn(move || {
            if let Some(device) = device {
                if let Ok(cfg) = device.default_input_config() {
                    let sample = cfg.sample_rate();
                    let channels = cfg.channels();
                    let config = cpal::SupportedStreamConfig::new(
                        if channels <= 2 { cfg.channels() } else { 2 },
                        sample,
                        *cfg.buffer_size(),
                        cpal::SampleFormat::F32,
                    );
                    if let Ok(mut resamp) =
                        FftFixedIn::<f32>::new(sample.0 as usize, SAMPLE_RATE, FRAME_SIZE, 8, 1)
                    {
                        let mut encoder =
                            Encoder::new(SAMPLE_RATE as u32, CHANNELS, Application::Audio).unwrap();
                        let mut extra = Vec::new();
                        match device.build_input_stream(
                            &config.into(),
                            move |data: &[f32], _| {
                                if channels == 1 {
                                    extra.extend(data);
                                } else {
                                    extra.extend(
                                        data.chunks(2)
                                            .map(|a| (a[0] + a[1]) * 0.5)
                                            .collect::<Vec<f32>>(),
                                    )
                                }
                                let mut v = Vec::new();
                                while extra.len() >= FRAME_SIZE {
                                    let mut compressed = vec![0u8; 1024];
                                    if let Ok(len) = encoder.encode_float(
                                        &resamp.process(&[&extra[..FRAME_SIZE]], None).unwrap()[0],
                                        &mut compressed,
                                    ) {
                                        if len != 0 {
                                            v.push(compressed[..len].to_vec())
                                        }
                                    }
                                    extra.drain(..FRAME_SIZE);
                                }
                                for v in v {
                                    let _ = tx.send(v);
                                }
                            },
                            |err| error!("Stream error: {}", err),
                            Some(Duration::from_millis(10)),
                        ) {
                            Ok(stream) => {
                                if let Ok(_s) = stream.play() {
                                    loop {
                                        thread::sleep(Duration::from_millis(10))
                                    }
                                } else {
                                    error!("failed to play stream")
                                }
                            }
                            Err(s) => {
                                error!(
                                    "no stream {}, {}, {}, {}",
                                    s,
                                    cfg.channels(),
                                    cfg.sample_rate().0,
                                    cfg.sample_format()
                                )
                            }
                        }
                    } else {
                        warn!("resamp not found")
                    }
                } else {
                    warn!("input config not found")
                }
            } else {
                warn!("input device not found")
            }
        });
        let stream_handle: Option<(OutputStream, OutputStreamHandle)> = {
            let output = audio.output_device.clone();
            if audio.disabled {
                None
            } else if output.is_none() {
                host.default_output_device()
            } else if let Some(d) = host
                .output_devices()
                .map(|mut d| d.find(|d| d.name().ok() == output))
                .ok()
                .flatten()
            {
                Some(d)
            } else {
                host.default_output_device()
            }
        }
        .and_then(|device| {
            device
                .default_output_config()
                .map(|config| OutputStream::try_from_device_config(&device, config).ok())
                .ok()
                .flatten()
        });
        let sink: HashMap<OmniPeerId, PlayerInfo> = Default::default();
        Self {
            decoder,
            stream_handle,
            per_player: sink,
            rx,
        }
    }

    pub fn recv_audio(&mut self) -> Result<Vec<u8>, TryRecvError> {
        self.rx.try_recv()
    }

    pub fn play_audio(
        &mut self,
        audio: AudioSettings,
        player_pos: (i32, i32),
        src: OmniPeerId,
        data: Vec<Vec<u8>>,
        global: bool,
        sound_pos: (i32, i32),
    ) {
        if let std::collections::hash_map::Entry::Vacant(e) = self.per_player.entry(src) {
            if let Some(stream_handle) = &self.stream_handle {
                if let Ok(s) = Sink::try_new(&stream_handle.1) {
                    //let (pitch_control, dsp) = make_dsp();
                    e.insert(PlayerInfo {
                        sink: s,
                        //tracker: VelocityTracker::default(),
                        //pitch_control,
                        //dsp,
                    });
                }
            }
        }
        self.per_player.entry(src).and_modify(|player_info| {
            /*player_info
            .tracker
            .push_new_pos(sound_pos.into(), player_pos.into());*/
            let vol = {
                if global {
                    *audio.volume.get(&src).unwrap_or(&1.0)
                } else {
                    let (mx, my) = (player_pos.0, player_pos.1);
                    let dx = mx.abs_diff(sound_pos.0) as u64;
                    let dy = my.abs_diff(sound_pos.1) as u64;
                    let dist = dx * dx + dy * dy;
                    if dist > audio.range.pow(2) {
                        0.0
                    } else {
                        audio.volume.get(&src).unwrap_or(&1.0)
                            / (1.0 + audio.dropoff.mul(dist as f32 * 2.0f32.powi(-18)))
                    }
                }
            };
            if vol > 0.0 && !audio.mute_out {
                player_info.sink.set_volume(vol);
                let mut dec: Vec<f32> = Vec::new();
                for data in data {
                    let mut out = vec![0f32; FRAME_SIZE];
                    if let Ok(len) = self.decoder.decode_float(&data, &mut out, false) {
                        if len != 0 {
                            dec.extend(&out[..len])
                        }
                    }
                }
                if !dec.is_empty() {
                    /*let speed = player_info.tracker.calc_speed_toward_player();
                    let pitch_change = (1.0 / (1.0 + speed / SPEED_OF_SOUND)).clamp(0.01, 3.00);
                    player_info.pitch_control.store(
                        (pitch_change * DSP_PITCH_MULT as f32) as usize,
                        std::sync::atomic::Ordering::Relaxed,
                    );

                    let mut dsp_out = vec![0f32; dec.len()];

                    player_info.dsp.process_big(
                        dec.len(),
                        &[dec.as_slice()],
                        &mut [dsp_out.as_mut_slice()],
                    );
                    let source = SamplesBuffer::new(1, SAMPLE_RATE as u32, dsp_out);*/
                    let source = SamplesBuffer::new(1, SAMPLE_RATE as u32, dec);
                    player_info.sink.append(source);
                    player_info.sink.play();
                }
            }
        });
    }
}
