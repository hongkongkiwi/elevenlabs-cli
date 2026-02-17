//! Audio utilities for real-time playback and recording
//!
//! This module provides cross-platform audio functionality when the "audio" feature is enabled.

#[cfg(feature = "audio")]
pub mod audio_io {
    use std::sync::mpsc::{self, Sender};
    use std::thread;

    /// Play audio bytes to the default output device (speaker)
    ///
    /// Uses rodio for cross-platform playback (macOS, Linux, Windows)
    pub fn play_to_speaker(data: &[u8]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use rodio::{Decoder, OutputStream, Sink};

        // Convert to owned data to avoid lifetime issues
        let owned_data = data.to_vec();
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let source = Decoder::new(std::io::Cursor::new(owned_data))?;
        let sink = Sink::try_new(&stream_handle)?;
        sink.append(source);
        sink.sleep_until_end();

        Ok(())
    }

    /// Streaming audio player that can receive chunks in real-time
    ///
    /// Create with `new()`, then call `send_chunk()` from async context,
    /// and `finish()` when done. Runs playback in separate thread.
    pub struct StreamingPlayer {
        sender: Option<Sender<Vec<u8>>>,
        handle: Option<thread::JoinHandle<()>>,
    }

    impl StreamingPlayer {
        /// Create a new streaming player and start playback thread
        pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
            use rodio::{Decoder, OutputStream, Sink};

            let (tx, rx) = mpsc::channel::<Vec<u8>>();

            let handle = thread::spawn(move || {
                let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                let sink = Sink::try_new(&stream_handle).unwrap();

                // Buffer for accumulating chunks
                let mut buffer = Vec::new();
                let min_chunk_size = 4096;

                loop {
                    // Try to receive with timeout
                    match rx.recv_timeout(std::time::Duration::from_millis(100)) {
                        Ok(chunk) => {
                            buffer.extend_from_slice(&chunk);

                            // Try to decode and play when we have enough data
                            if buffer.len() >= min_chunk_size {
                                // Clone buffer for playback
                                let playback_data = buffer.clone();
                                if let Ok(source) =
                                    Decoder::new(std::io::Cursor::new(playback_data))
                                {
                                    sink.append(source);
                                }
                                // Keep some data in buffer for continuity
                                let keep = buffer.len() / 2;
                                if keep > 0 && keep < buffer.len() {
                                    buffer.drain(..keep);
                                }
                            }
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            // Try to play any remaining buffer
                            if !buffer.is_empty() {
                                let playback_data = buffer.clone();
                                if let Ok(source) =
                                    Decoder::new(std::io::Cursor::new(playback_data))
                                {
                                    sink.append(source);
                                }
                                buffer.clear();
                            }
                            // Check if we should exit (sink empty and channel disconnected)
                            if sink.empty() && rx.try_recv().is_err() {
                                break;
                            }
                        }
                        Err(mpsc::RecvTimeoutError::Disconnected) => {
                            // Play remaining and exit
                            if !buffer.is_empty() {
                                let playback_data = buffer.clone();
                                if let Ok(source) =
                                    Decoder::new(std::io::Cursor::new(playback_data))
                                {
                                    sink.append(source);
                                }
                            }
                            break;
                        }
                    }
                }

                sink.sleep_until_end();
            });

            Ok(Self {
                sender: Some(tx),
                handle: Some(handle),
            })
        }

        /// Send an audio chunk to be played
        pub fn send_chunk(
            &self,
            chunk: &[u8],
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            if let Some(ref sender) = self.sender {
                sender.send(chunk.to_vec())?;
            }
            Ok(())
        }

        /// Signal that no more chunks will be sent and wait for playback to finish
        pub fn finish(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            drop(self.sender);
            if let Some(handle) = self.handle {
                handle
                    .join()
                    .map_err(|e| format!("Thread panicked: {:?}", e))?;
            }
            Ok(())
        }
    }

    /// Record audio from the default input device (microphone)
    ///
    /// Returns raw PCM audio data. Duration is in seconds.
    pub fn record_from_microphone(
        duration_secs: f32,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
        use cpal::SampleFormat;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No input device available")?;

        let config = device
            .default_input_config()
            .map_err(|e| format!("Failed to get default input config: {}", e))?;

        let sample_rate = config.sample_rate().0 as f32;
        let channels = config.channels() as usize;
        let samples = (duration_secs * sample_rate * channels as f32) as usize;

        let buffer = Arc::new(std::sync::Mutex::new(Vec::with_capacity(samples * 2)));
        let buffer_clone = buffer.clone();
        let recording = Arc::new(AtomicBool::new(true));
        let recording_clone = recording.clone();

        let stream = match config.sample_format() {
            SampleFormat::I16 => {
                let buf = buffer_clone;
                let rec = recording_clone;
                device.build_input_stream(
                    &config.into(),
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        if rec.load(Ordering::Relaxed) {
                            if let Ok(mut buf) = buf.lock() {
                                for &sample in data {
                                    buf.push(sample as u8);
                                    buf.push((sample >> 8) as u8);
                                }
                            }
                        }
                    },
                    |err| eprintln!("Error in audio stream: {}", err),
                    None,
                )?
            }
            SampleFormat::F32 => {
                let buf = buffer_clone;
                let rec = recording_clone;
                device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        if rec.load(Ordering::Relaxed) {
                            if let Ok(mut buf) = buf.lock() {
                                for &sample in data {
                                    let sample_i16 = (sample * i16::MAX as f32) as i16;
                                    buf.push(sample_i16 as u8);
                                    buf.push((sample_i16 >> 8) as u8);
                                }
                            }
                        }
                    },
                    |err| eprintln!("Error in audio stream: {}", err),
                    None,
                )?
            }
            _ => return Err("Unsupported sample format".into()),
        };

        stream.play()?;

        std::thread::sleep(std::time::Duration::from_secs_f32(duration_secs));
        recording.store(false, Ordering::Relaxed);
        drop(stream);

        let buffer = buffer.lock().unwrap();
        Ok(buffer.clone())
    }

    /// Stream from microphone to a callback function
    ///
    /// The callback receives audio chunks as they're recorded
    #[allow(dead_code)]
    pub fn stream_from_microphone(
        _chunk_duration_secs: f32,
        _callback: impl FnMut(&[u8]) + Send + 'static,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // For simplicity, just use record_from_microphone with the full duration
        // and call the callback once with all the data
        Err("stream_from_microphone not yet implemented - use record_from_microphone".into())
    }

    /// Record from microphone with Voice Activity Detection (VAD)
    ///
    /// Automatically stops recording when speech ends.
    /// - `max_duration`: Maximum recording time in seconds
    /// - `silence_threshold`: Energy threshold below which is considered silence (0.0-1.0)
    /// - `silence_duration`: How many seconds of silence before stopping
    #[allow(dead_code)]
    pub fn record_with_vad(
        max_duration_secs: f32,
        silence_threshold: f32,
        silence_duration_secs: f32,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
        use cpal::SampleFormat;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        use std::time::Instant;

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No input device available")?;

        let config = device
            .default_input_config()
            .map_err(|e| format!("Failed to get default input config: {}", e))?;

        let sample_rate = config.sample_rate().0 as f32;
        let channels = config.channels() as usize;
        let max_samples = (max_duration_secs * sample_rate * channels as f32) as usize;

        let buffer = Arc::new(std::sync::Mutex::new(Vec::with_capacity(max_samples * 2)));
        let buffer_clone = buffer.clone();

        let recording = Arc::new(AtomicBool::new(true));
        let recording_clone = recording.clone();

        // VAD state
        let silence_start = Arc::new(std::sync::Mutex::new(Option::<Instant>::None));
        let silence_start_clone = silence_start.clone();

        let chunk_sample_count = (sample_rate * 0.1) as usize; // ~100ms chunks
        let _silence_samples_needed =
            (silence_duration_secs * sample_rate * channels as f32) as usize;

        let stream = match config.sample_format() {
            SampleFormat::I16 => {
                let buf = buffer_clone;
                let rec = recording_clone;
                let silence = silence_start_clone;
                device.build_input_stream(
                    &config.into(),
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        if rec.load(Ordering::Relaxed) {
                            if let Ok(mut buf) = buf.lock() {
                                // Calculate RMS energy
                                let sum: i64 = data.iter().map(|&s| (s as i64) * (s as i64)).sum();
                                let rms = (sum as f64 / data.len() as f64).sqrt();
                                let normalized = (rms / i16::MAX as f64).min(1.0) as f32;

                                // Record audio
                                for &sample in data {
                                    buf.push(sample as u8);
                                    buf.push((sample >> 8) as u8);
                                }

                                // VAD logic
                                let mut silence_guard = silence.lock().unwrap();
                                if normalized > silence_threshold {
                                    *silence_guard = None; // Voice detected
                                } else if buf.len() > chunk_sample_count * 2 {
                                    // Silence - check duration
                                    if silence_guard.is_none() {
                                        *silence_guard = Some(Instant::now());
                                    } else if let Some(start) = *silence_guard {
                                        let elapsed = start.elapsed().as_secs_f32();
                                        if elapsed >= silence_duration_secs {
                                            rec.store(false, Ordering::Relaxed);
                                            // Stop recording
                                        }
                                    }
                                }
                            }
                        }
                    },
                    |err| eprintln!("Error in audio stream: {}", err),
                    None,
                )?
            }
            SampleFormat::F32 => {
                let buf = buffer_clone;
                let rec = recording_clone;
                let silence = silence_start_clone;
                device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        if rec.load(Ordering::Relaxed) {
                            if let Ok(mut buf) = buf.lock() {
                                // Calculate RMS energy
                                let sum: f64 = data.iter().map(|&s| (s as f64) * (s as f64)).sum();
                                let rms = (sum / data.len() as f64).sqrt();
                                let normalized = rms.min(1.0) as f32;

                                // Record audio
                                for &sample in data {
                                    let sample_i16 = (sample * i16::MAX as f32) as i16;
                                    buf.push(sample_i16 as u8);
                                    buf.push((sample_i16 >> 8) as u8);
                                }

                                // VAD logic
                                let mut silence_guard = silence.lock().unwrap();
                                if normalized > silence_threshold {
                                    *silence_guard = None;
                                } else if buf.len() > chunk_sample_count * 2 {
                                    if silence_guard.is_none() {
                                        *silence_guard = Some(Instant::now());
                                    } else if let Some(start) = *silence_guard {
                                        let elapsed = start.elapsed().as_secs_f32();
                                        if elapsed >= silence_duration_secs {
                                            rec.store(false, Ordering::Relaxed);
                                        }
                                    }
                                }
                            }
                        }
                    },
                    |err| eprintln!("Error in audio stream: {}", err),
                    None,
                )?
            }
            _ => return Err("Unsupported sample format".into()),
        };

        stream.play()?;

        // Wait for VAD to stop recording or max duration
        while recording.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(100));
            // Check max duration
            if buffer.lock().unwrap().len() >= max_samples * 2 {
                recording.store(false, Ordering::Relaxed);
            }
        }

        drop(stream);

        let buffer = buffer.lock().unwrap();
        Ok(buffer.clone())
    }

    /// Stream from microphone with callback and optional VAD
    ///
    /// - `chunk_duration`: How often to call callback (in seconds)
    /// - `callback`: Called with each chunk of audio
    /// - `use_vad`: If true, stops streaming when speech ends
    /// - `silence_threshold`: Energy threshold for VAD (0.0-1.0)
    /// - `silence_duration`: Seconds of silence before stopping
    #[allow(dead_code)]
    pub fn stream_from_microphone_with_vad(
        chunk_duration_secs: f32,
        mut callback: impl FnMut(&[u8]) + Send + 'static,
        use_vad: bool,
        silence_threshold: f32,
        silence_duration_secs: f32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
        use cpal::SampleFormat;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        use std::time::Instant;

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No input device available")?;

        let config = device
            .default_input_config()
            .map_err(|e| format!("Failed to get default input config: {}", e))?;

        let sample_rate = config.sample_rate().0 as f32;
        let channels = config.channels() as usize;
        let chunk_size = (chunk_duration_secs * sample_rate * channels as f32) as usize * 2;

        let chunk_buffer = Arc::new(std::sync::Mutex::new(Vec::with_capacity(chunk_size)));
        let chunk_buffer_clone = chunk_buffer.clone();
        let streaming = Arc::new(AtomicBool::new(true));
        let streaming_clone = streaming.clone();

        let silence_start = Arc::new(std::sync::Mutex::new(None::<Instant>));
        let silence_start_clone = silence_start.clone();

        let stream = match config.sample_format() {
            SampleFormat::I16 => {
                let buf = chunk_buffer_clone;
                let stream_flag = streaming_clone;
                let silence = silence_start_clone;

                device.build_input_stream(
                    &config.into(),
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        if stream_flag.load(Ordering::Relaxed) {
                            if let Ok(mut buf) = buf.lock() {
                                // Calculate energy
                                let sum: i64 = data.iter().map(|&s| (s as i64) * (s as i64)).sum();
                                let rms = (sum as f64 / data.len() as f64).sqrt();
                                let normalized = (rms / i16::MAX as f64).min(1.0) as f32;

                                // Record to chunk buffer
                                for &sample in data {
                                    buf.push(sample as u8);
                                    buf.push((sample >> 8) as u8);
                                }

                                // VAD check
                                if use_vad {
                                    let mut silence_guard = silence.lock().unwrap();
                                    if normalized > silence_threshold {
                                        *silence_guard = None;
                                    } else if buf.len() >= chunk_size {
                                        if silence_guard.is_none() {
                                            *silence_guard = Some(Instant::now());
                                        } else if let Some(start) = *silence_guard {
                                            if start.elapsed().as_secs_f32()
                                                >= silence_duration_secs
                                            {
                                                stream_flag.store(false, Ordering::Relaxed);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    |err| eprintln!("Error in audio stream: {}", err),
                    None,
                )?
            }
            SampleFormat::F32 => {
                let buf = chunk_buffer_clone;
                let stream_flag = streaming_clone;
                let silence = silence_start_clone;

                device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        if stream_flag.load(Ordering::Relaxed) {
                            if let Ok(mut buf) = buf.lock() {
                                let sum: f64 = data.iter().map(|&s| (s as f64) * (s as f64)).sum();
                                let rms = (sum / data.len() as f64).sqrt();
                                let normalized = rms.min(1.0) as f32;

                                for &sample in data {
                                    let sample_i16 = (sample * i16::MAX as f32) as i16;
                                    buf.push(sample_i16 as u8);
                                    buf.push((sample_i16 >> 8) as u8);
                                }

                                if use_vad {
                                    let mut silence_guard = silence.lock().unwrap();
                                    if normalized > silence_threshold {
                                        *silence_guard = None;
                                    } else if buf.len() >= chunk_size {
                                        if silence_guard.is_none() {
                                            *silence_guard = Some(Instant::now());
                                        } else if let Some(start) = *silence_guard {
                                            if start.elapsed().as_secs_f32()
                                                >= silence_duration_secs
                                            {
                                                stream_flag.store(false, Ordering::Relaxed);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    |err| eprintln!("Error in audio stream: {}", err),
                    None,
                )?
            }
            _ => return Err("Unsupported sample format".into()),
        };

        stream.play()?;

        let chunk_interval = std::time::Duration::from_secs_f32(chunk_duration_secs);

        while streaming.load(Ordering::Relaxed) {
            std::thread::sleep(chunk_interval);
            let mut chunk = chunk_buffer.lock().unwrap();
            if !chunk.is_empty() {
                callback(&chunk);
                chunk.clear();
            }
        }

        // Final chunk
        {
            let chunk = chunk_buffer.lock().unwrap();
            if !chunk.is_empty() {
                callback(&chunk);
            }
        }

        drop(stream);
        Ok(())
    }

    /// List available audio input devices
    #[allow(dead_code)]
    pub fn list_input_devices() -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        use cpal::traits::{DeviceTrait, HostTrait};

        let host = cpal::default_host();
        let mut devices = Vec::new();

        for device in host.input_devices()? {
            if let Ok(name) = device.name() {
                devices.push(name);
            }
        }

        Ok(devices)
    }

    /// List available audio output devices
    #[allow(dead_code)]
    pub fn list_output_devices() -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        use cpal::traits::{DeviceTrait, HostTrait};

        let host = cpal::default_host();
        let mut devices = Vec::new();

        for device in host.output_devices()? {
            if let Ok(name) = device.name() {
                devices.push(name);
            }
        }

        Ok(devices)
    }

    /// Get input device by name (partial match)
    #[allow(dead_code)]
    pub fn get_input_device(
        name: &str,
    ) -> Result<Option<cpal::Device>, Box<dyn std::error::Error + Send + Sync>> {
        use cpal::traits::{DeviceTrait, HostTrait};

        let host = cpal::default_host();

        for device in host.input_devices()? {
            if let Ok(device_name) = device.name() {
                if device_name.to_lowercase().contains(&name.to_lowercase()) {
                    return Ok(Some(device));
                }
            }
        }

        Ok(None)
    }

    /// Get output device by name (partial match)
    #[allow(dead_code)]
    pub fn get_output_device(
        name: &str,
    ) -> Result<Option<cpal::Device>, Box<dyn std::error::Error + Send + Sync>> {
        use cpal::traits::{DeviceTrait, HostTrait};

        let host = cpal::default_host();

        for device in host.output_devices()? {
            if let Ok(device_name) = device.name() {
                if device_name.to_lowercase().contains(&name.to_lowercase()) {
                    return Ok(Some(device));
                }
            }
        }

        Ok(None)
    }
}

#[cfg(not(feature = "audio"))]
pub mod audio_io {
    /// Stub for play_to_speaker
    pub fn play_to_speaker(_data: &[u8]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Err("Audio feature not enabled. Rebuild with --features audio".into())
    }

    /// Stub for StreamingPlayer
    #[allow(dead_code)]
    pub struct StreamingPlayer;

    /// Stub for StreamingPlayer::new()
    #[allow(dead_code)]
    impl StreamingPlayer {
        pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
            Err("Audio feature not enabled. Rebuild with --features audio".into())
        }
        pub fn send_chunk(
            &self,
            _chunk: &[u8],
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Err("Audio feature not enabled".into())
        }
        pub fn finish(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Err("Audio feature not enabled".into())
        }
    }

    /// Stub for record_from_microphone
    pub fn record_from_microphone(
        _duration_secs: f32,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        Err("Audio feature not enabled. Rebuild with --features audio".into())
    }

    /// Stub for stream_from_microphone
    pub fn stream_from_microphone(
        _chunk_duration_secs: f32,
        _callback: impl FnMut(&[u8]) + Send + 'static,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Err("Audio feature not enabled. Rebuild with --features audio".into())
    }

    /// Stub for record_with_vad
    #[allow(dead_code)]
    pub fn record_with_vad(
        _max_duration_secs: f32,
        _silence_threshold: f32,
        _silence_duration_secs: f32,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        Err("Audio feature not enabled. Rebuild with --features audio".into())
    }

    /// Stub for stream_from_microphone_with_vad
    #[allow(dead_code)]
    pub fn stream_from_microphone_with_vad(
        _chunk_duration_secs: f32,
        _callback: impl FnMut(&[u8]) + Send + 'static,
        _use_vad: bool,
        _silence_threshold: f32,
        _silence_duration_secs: f32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Err("Audio feature not enabled. Rebuild with --features audio".into())
    }

    /// Stub for list_input_devices
    #[allow(dead_code)]
    pub fn list_input_devices() -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        Err("Audio feature not enabled. Rebuild with --features audio".into())
    }

    /// Stub for list_output_devices
    #[allow(dead_code)]
    pub fn list_output_devices() -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        Err("Audio feature not enabled. Rebuild with --features audio".into())
    }

    /// Stub for get_input_device
    #[allow(dead_code)]
    pub fn get_input_device(
        _name: &str,
    ) -> Result<Option<cpal::Device>, Box<dyn std::error::Error + Send + Sync>> {
        Err("Audio feature not enabled. Rebuild with --features audio".into())
    }

    /// Stub for get_output_device
    #[allow(dead_code)]
    pub fn get_output_device(
        _name: &str,
    ) -> Result<Option<cpal::Device>, Box<dyn std::error::Error + Send + Sync>> {
        Err("Audio feature not enabled. Rebuild with --features audio".into())
    }
}
