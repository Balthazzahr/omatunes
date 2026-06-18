use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SampleRate, StreamConfig};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::Time as SymphoniaTime;
use tokio::sync::mpsc;

const OUTPUT_RATE: u32 = 48000;
const OUTPUT_CHANNELS: u16 = 2;

fn sync_system_volume(volume: f32) {
    let vol_pct = format!("{}%", (volume * 100.0).round() as u32);
    std::thread::spawn(move || {
        for delay in &[10, 50, 150, 300, 600] {
            std::thread::sleep(std::time::Duration::from_millis(*delay));
            if let Ok(out) = std::process::Command::new("pactl").arg("list").arg("sink-inputs").output() {
                let text = String::from_utf8_lossy(&out.stdout);
                let mut current_id = None;
                let mut found = false;
                for line in text.lines() {
                    let line = line.trim();
                    if line.starts_with("Sink Input #") {
                        if let Some(id_str) = line.strip_prefix("Sink Input #") {
                            current_id = id_str.parse::<u32>().ok();
                        }
                    } else if line.contains("omatunes") {
                        if let Some(id) = current_id {
                            let _ = std::process::Command::new("pactl")
                                .arg("set-sink-input-volume")
                                .arg(id.to_string())
                                .arg(&vol_pct)
                                .spawn();
                            found = true;
                        }
                    }
                }
                if found {
                    break;
                }
            }
        }
    });
}

#[derive(Debug, Clone)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug)]
pub enum AudioCommand {
    Play(PathBuf),
    Pause,
    Resume,
    Stop,
    Seek(Duration),
    SetVolume(f32),
    Next,
    Previous,
}

#[derive(Debug, Clone)]
pub enum AudioEvent {
    Playing { path: PathBuf },
    Paused,
    Stopped,
    Progress { position: Duration, duration: Duration },
    Error(String),
    TrackEnded,
}

pub struct AudioPlayer {
    pub cmd_tx: mpsc::UnboundedSender<AudioCommand>,
    pub event_rx: mpsc::UnboundedReceiver<AudioEvent>,
    pub sample_buffer: Arc<Mutex<VecDeque<f32>>>,
}

impl AudioPlayer {
    pub fn spawn() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let sample_buffer: Arc<Mutex<VecDeque<f32>>> =
            Arc::new(Mutex::new(VecDeque::with_capacity(8192)));

        let buf_clone = sample_buffer.clone();
        std::thread::spawn(move || audio_thread(cmd_rx, event_tx, buf_clone));

        AudioPlayer { cmd_tx, event_rx, sample_buffer }
    }

    pub fn send(&self, cmd: AudioCommand) {
        let _ = self.cmd_tx.send(cmd);
    }
}

// ── Thread de áudio ──────────────────────────────────────────────────────────

fn audio_thread(
    mut cmd_rx: mpsc::UnboundedReceiver<AudioCommand>,
    event_tx: mpsc::UnboundedSender<AudioEvent>,
    sample_buffer: Arc<Mutex<VecDeque<f32>>>,
) {
    let host   = cpal::default_host();
    let device = match host.default_output_device() {
        Some(d) => d,
        None => {
            let _ = event_tx.send(AudioEvent::Error("Nenhum dispositivo de áudio".into()));
            return;
        }
    };

    let sample_format = device
        .default_output_config()
        .map(|c| c.sample_format())
        .unwrap_or(SampleFormat::F32);

    let stream_config = StreamConfig {
        channels:    OUTPUT_CHANNELS,
        sample_rate: SampleRate(OUTPUT_RATE),
        buffer_size: cpal::BufferSize::Default,
    };

    let pcm: Arc<Mutex<VecDeque<f32>>> =
        Arc::new(Mutex::new(VecDeque::with_capacity(OUTPUT_RATE as usize * 2)));

    // Compartilhados entre fill_output e o loop de comandos
    let paused: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let shared_vol: Arc<Mutex<f32>> = Arc::new(Mutex::new(0.8));

    let pcm_cb     = pcm.clone();
    let paused_cb  = paused.clone();
    let vol_cb     = shared_vol.clone();
    let vis_cb     = sample_buffer.clone();
    let err_fn     = |e| eprintln!("Erro stream: {e}");

    let stream = match sample_format {
        SampleFormat::I16 => {
            let pcm2     = pcm.clone();
            let paused2  = paused.clone();
            let vol2     = shared_vol.clone();
            let vis2     = sample_buffer.clone();
            device.build_output_stream(
                &stream_config,
                move |data: &mut [i16], _| {
                    let mut tmp = vec![0f32; data.len()];
                    fill_output(&mut tmp, &pcm2, &paused2, &vol2, &vis2);
                    for (d, s) in data.iter_mut().zip(tmp.iter()) {
                        *d = cpal::Sample::from_sample(*s);
                    }
                },
                err_fn, None,
            )
        }
        _ => device.build_output_stream(
            &stream_config,
            move |data: &mut [f32], _| fill_output(data, &pcm_cb, &paused_cb, &vol_cb, &vis_cb),
            err_fn, None,
        ),
    };

    let stream = match stream {
        Ok(s)  => s,
        Err(e) => {
            let _ = event_tx.send(AudioEvent::Error(format!("Build stream: {e}")));
            return;
        }
    };

    if let Err(e) = stream.play() {
        let _ = event_tx.send(AudioEvent::Error(format!("Stream play: {e}")));
        return;
    }

    let mut cancel: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let mut current_path: Option<PathBuf> = None;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        loop {
            match cmd_rx.recv().await {
                None => break,

                Some(AudioCommand::Stop) => {
                    cancel.store(true, Ordering::SeqCst);
                    paused.store(false, Ordering::SeqCst);
                    pcm.lock().unwrap().clear();
                    current_path = None;
                    let _ = event_tx.send(AudioEvent::Stopped);
                }

                Some(AudioCommand::Pause) => {
                    paused.store(true, Ordering::SeqCst);
                    let _ = event_tx.send(AudioEvent::Paused);
                }

                Some(AudioCommand::Resume) => {
                    paused.store(false, Ordering::SeqCst);
                    if let Some(ref path) = current_path {
                        let _ = event_tx.send(AudioEvent::Playing { path: path.clone() });
                    }
                }

                Some(AudioCommand::SetVolume(v)) => {
                    let clamped = v.clamp(0.0, 1.0);
                    *shared_vol.lock().unwrap() = clamped;
                    sync_system_volume(clamped);
                }

                Some(AudioCommand::Seek(pos)) => {
                    if let Some(path) = current_path.clone() {
                        cancel.store(true, Ordering::SeqCst);
                        paused.store(false, Ordering::SeqCst);
                        pcm.lock().unwrap().clear();

                        let new_cancel = Arc::new(AtomicBool::new(false));
                        cancel = new_cancel.clone();

                        let pcm2 = pcm.clone();
                        let tx   = event_tx.clone();
                        let vol  = shared_vol.clone();
                        let flag = new_cancel;

                        tokio::task::spawn_blocking(move || {
                            match decode_file(&path, pcm2, tx.clone(), vol, flag, Some(pos)) {
                                Ok(true)  => { let _ = tx.send(AudioEvent::TrackEnded); }
                                Ok(false) => {}
                                Err(e)    => { let _ = tx.send(AudioEvent::Error(e.to_string())); }
                            }
                        });
                    }
                }

                Some(AudioCommand::Play(path)) => {
                    cancel.store(true, Ordering::SeqCst);
                    paused.store(false, Ordering::SeqCst);
                    pcm.lock().unwrap().clear();

                    let new_cancel = Arc::new(AtomicBool::new(false));
                    cancel = new_cancel.clone();
                    current_path = Some(path.clone());

                    let _ = event_tx.send(AudioEvent::Playing { path: path.clone() });

                    let pcm2 = pcm.clone();
                    let tx   = event_tx.clone();
                    let vol  = shared_vol.clone();
                    let flag = new_cancel;

                    let vol_val = *vol.lock().unwrap();
                    sync_system_volume(vol_val);

                    tokio::task::spawn_blocking(move || {
                        match decode_file(&path, pcm2, tx.clone(), vol, flag, None) {
                            Ok(true)  => { let _ = tx.send(AudioEvent::TrackEnded); }
                            Ok(false) => {}
                            Err(e)    => { let _ = tx.send(AudioEvent::Error(e.to_string())); }
                        }
                    });
                }

                Some(_) => {}
            }
        }
    });
}

fn fill_output(
    output: &mut [f32],
    pcm: &Arc<Mutex<VecDeque<f32>>>,
    paused: &Arc<AtomicBool>,
    volume: &Arc<Mutex<f32>>,
    vis_buffer: &Arc<Mutex<VecDeque<f32>>>,
) {
    if paused.load(Ordering::SeqCst) {
        for s in output.iter_mut() { *s = 0.0; }
        return;
    }
    let vol = *volume.lock().unwrap();
    let mut buf = pcm.lock().unwrap();
    let mut vis = vis_buffer.lock().unwrap();
    for sample in output.iter_mut() {
        let s = buf.pop_front().unwrap_or(0.0) * vol;
        *sample = s;
        vis.push_back(s);
        if vis.len() > 8192 {
            vis.pop_front();
        }
    }
}

// ── Decode ───────────────────────────────────────────────────────────────────

/// Retorna Ok(true) se a faixa terminou normalmente, Ok(false) se foi cancelada.
fn decode_file(
    path: &PathBuf,
    pcm: Arc<Mutex<VecDeque<f32>>>,
    event_tx: mpsc::UnboundedSender<AudioEvent>,
    _volume: Arc<Mutex<f32>>,
    cancel: Arc<AtomicBool>,
    seek_to: Option<Duration>,
) -> Result<bool> {
    let file = std::fs::File::open(path)?;
    let mss  = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let fmt_opts = FormatOptions { enable_gapless: true, ..Default::default() };

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &MetadataOptions::default())
        .map_err(|e| anyhow!("Formato não suportado: {e}"))?;

    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| anyhow!("Nenhuma faixa de áudio"))?;

    let track_id  = track.id;
    let time_base = track.codec_params.time_base;
    let n_frames  = track.codec_params.n_frames;
    let file_rate = track.codec_params.sample_rate.unwrap_or(44100);

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| anyhow!("Decoder: {e}"))?;

    let mut sample_count = if let Some(pos) = seek_to {
        let seek_time = SymphoniaTime {
            seconds: pos.as_secs(),
            frac: pos.subsec_nanos() as f64 / 1_000_000_000.0,
        };
        format.seek(SeekMode::Accurate, SeekTo::Time { time: seek_time, track_id: None }).ok();
        decoder.reset();
        (pos.as_secs_f64() * OUTPUT_RATE as f64) as u64
    } else {
        0u64
    };

    loop {
        if cancel.load(Ordering::SeqCst) {
            return Ok(false);
        }

        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(SymphoniaError::IoError(_)) => break,
            Err(SymphoniaError::ResetRequired) => { decoder.reset(); continue; }
            Err(e) => return Err(anyhow!("Packet: {e}")),
        };

        if packet.track_id() != track_id { continue; }

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(e) => return Err(anyhow!("Decode: {e}")),
        };

        let spec       = *decoded.spec();
        let n_channels = spec.channels.count();

        let mut conv = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
        conv.copy_interleaved_ref(decoded);
        let raw = conv.samples();


        let stereo: Vec<f32> = match n_channels {
            1 => raw.iter().flat_map(|&s| [s, s]).collect(),
            2 => raw.to_vec(),
            n => raw.chunks(n).flat_map(|ch| {
                let l = ch.first().copied().unwrap_or(0.0);
                let r = ch.get(1).copied().unwrap_or(0.0);
                [l, r]
            }).collect(),
        };

        let samples = if file_rate != OUTPUT_RATE {
            resample_stereo(&stereo, file_rate, OUTPUT_RATE)
        } else {
            stereo
        };

        sample_count += samples.len() as u64 / 2;

        let position = Duration::from_secs_f64(sample_count as f64 / OUTPUT_RATE as f64);
        let duration = if let (Some(tb), Some(nf)) = (time_base, n_frames) {
            Duration::from_secs_f64(nf as f64 * tb.numer as f64 / tb.denom as f64)
        } else {
            // Opus/OGG: duration unknown until EOF, mirror position so UI stays responsive
            position
        };
        let _ = event_tx.send(AudioEvent::Progress { position, duration });

        loop {
            if cancel.load(Ordering::SeqCst) {
                return Ok(false);
            }
            if pcm.lock().unwrap().len() < OUTPUT_RATE as usize * 2 {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }

        pcm.lock().unwrap().extend(samples);
    }

    Ok(true)
}

fn resample_stereo(input: &[f32], in_rate: u32, out_rate: u32) -> Vec<f32> {
    let ratio      = in_rate as f64 / out_rate as f64;
    let in_frames  = input.len() / 2;
    let out_frames = (in_frames as f64 / ratio).ceil() as usize;
    let mut out    = Vec::with_capacity(out_frames * 2);

    for i in 0..out_frames {
        let src  = i as f64 * ratio;
        let idx  = src as usize;
        let frac = (src - idx as f64) as f32;

        let l0 = input.get(idx * 2    ).copied().unwrap_or(0.0);
        let l1 = input.get(idx * 2 + 2).copied().unwrap_or(l0);
        let r0 = input.get(idx * 2 + 1).copied().unwrap_or(0.0);
        let r1 = input.get(idx * 2 + 3).copied().unwrap_or(r0);

        out.push(l0 + (l1 - l0) * frac);
        out.push(r0 + (r1 - r0) * frac);
    }

    out
}
