//! Audio utilities for real-time playback and recording
//!
//! This module provides cross-platform audio functionality when the "audio" feature is enabled.

#[cfg(feature = "audio")]
pub mod audio_io {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    /// Play audio bytes to the default output device (speaker)
    /// 
    /// Uses rodio for cross-platform playback (macOS, Linux, Windows)
    pub fn play_to_speaker(data: &[u8]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use rodio::{Decoder, OutputStream, Sink};

        let (_stream, stream_handle) = OutputStream::try_default()?;
        let source = Decoder::new(std::io::Cursor::new(data))?;
        let sink = Sink::try_new(&stream_handle)?;
        sink.append(source);
        sink.sleep_until_end();
        
        Ok(())
    }

    /// Stream audio chunks to speaker in real-time
    /// 
    /// For use with streaming TTS APIs
    pub fn stream_to_speaker(
        chunk_stream: impl futures_util::Stream<Item = Result<bytes::Bytes, Box<dyn std::error::Error + Send + Sync>>> + Send,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use futures_util::pin_mut;
        use bytes::{BufMut, BytesMut};
        use rodio::{Decoder, OutputStream, Sink};

        pin_mut!(chunk_stream);

        let audio_output = OutputStream::try_default()?;
        let audio_sink = Sink::try_new(&audio_output.1)?;

        let mut buffer = BytesMut::with_capacity(16384);
        let min_chunk_size = 8192;

        while let Some(chunk_result) = chunk_stream.next().await {
            let chunk = chunk_result?;
            buffer.put(chunk);

            if buffer.len() >= min_chunk_size {
                let data = buffer.clone().freeze();
                if let Ok(source) = Decoder::new(std::io::Cursor::new(&data)) {
                    audio_sink.append(source);
                }
                buffer.clear();
            }
        }

        if !buffer.is_empty() {
            let data = buffer.freeze();
            if let Ok(source) = Decoder::new(std::io::Cursor::new(&data)) {
                audio_sink.append(source);
            }
        }

        audio_sink.sleep_until_end();
        Ok(())
    }

    /// Record audio from the default input device (microphone)
    /// 
    /// Returns raw PCM audio data. Duration is in seconds.
    pub fn record_from_microphone(
        duration_secs: f32,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
        use cpal::{Host, SampleFormat};

        let host = Host::default();
        let device = host.default_input_device()
            .ok_or("No input device available")?;
        
        let config = device.default_input_config()
            .map_err(|e| format!("Failed to get default input config: {}", e))?;

        let sample_rate = config.sample_rate().0 as f32;
        let channels = config.channels() as usize;
        let samples = (duration_secs * sample_rate * channels as f32) as usize;

        let err_fn = |err| eprintln!("Error in audio stream: {}", err);

        let buffer = Arc::new(std::sync::Mutex::new(Vec::with_capacity(samples * 2)));
        let buffer_clone = buffer.clone();
        let recording = Arc::new(AtomicBool::new(true));
        let recording_clone = recording.clone();

        let stream = match config.sample_format() {
            SampleFormat::I16 => {
                let data = buffer_clone;
                device.build_input_stream(
                    &config.into(),
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        if recording_clone.load(Ordering::Relaxed) {
                            let mut buf = data.lock().unwrap();
                            for &sample in data {
                                buf.push(sample as u8);
                                buf.push((sample >> 8) as u8);
                            }
                        }
                    },
                    err_fn,
                    None,
                )?
            }
            SampleFormat::F32 => {
                let data = buffer_clone;
                device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        if recording_clone.load(Ordering::Relaxed) {
                            let mut buf = data.lock().unwrap();
                            for &sample in data {
                                let sample_i16 = (sample * i16::MAX as f32) as i16;
                                buf.push(sample_i16 as u8);
                                buf.push((sample_i16 >> 8) as u8);
                            }
                        }
                    },
                    err_fn,
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
    pub fn stream_from_microphone(
        chunk_duration_secs: f32,
        mut callback: impl FnMut(&[u8]) + Send + 'static,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
        use cpal::{Host, SampleFormat};

        let host = Host::default();
        let device = host.default_input_device()
            .ok_or("No input device available")?;
        
        let config = device.default_input_config()
            .map_err(|e| format!("Failed to get default input config: {}", e))?;

        let sample_rate = config.sample_rate().0 as f32;
        let channels = config.channels() as usize;
        let chunk_size = (chunk_duration_secs * sample_rate * channels) as usize * 2;

        let chunk_buffer = Arc::new(std::sync::Mutex::new(Vec::with_capacity(chunk_size)));
        let chunk_buffer_clone = chunk_buffer.clone();
        let recording = Arc::new(AtomicBool::new(true));
        let recording_clone = recording.clone();

        let err_fn = |err| eprintln!("Error in audio stream: {}", err);

        let stream = match config.sample_format() {
            SampleFormat::I16 => {
                let buf = chunk_buffer_clone;
                let rec = recording_clone;
                device.build_input_stream(
                    &config.into(),
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        if rec.load(Ordering::Relaxed) {
                            let mut buf = buf.lock().unwrap();
                            for &sample in data {
                                buf.push(sample as u8);
                                buf.push((sample >> 8) as u8);
                                
                                if buf.len() >= chunk_size {
                                    let chunk = buf.clone();
                                    drop(buf);
                                    let mut buf = buf.lock().unwrap();
                                    callback(&chunk);
                                    buf.clear();
                                }
                            }
                        }
                    },
                    err_fn,
                    None,
                )?
            }
            SampleFormat::F32 => {
                let buf = chunk_buffer_clone;
                let rec = recording_clone;
                device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        if rec.load(Ordering::Relaxed) {
                            let mut buf = buf.lock().unwrap();
                            for &sample in data {
                                let sample_i16 = (sample * i16::MAX as f32) as i16;
                                buf.push(sample_i16 as u8);
                                buf.push((sample_i16 >> 8) as u8);
                                
                                if buf.len() >= chunk_size {
                                    let chunk = buf.clone();
                                    drop(buf);
                                    let mut buf = buf.lock().unwrap();
                                    callback(&chunk);
                                    buf.clear();
                                }
                            }
                        }
                    },
                    err_fn,
                    None,
                )?
            }
            _ => return Err("Unsupported sample format".into()),
        };

        stream.play()?;
        
        // Stream until manually stopped
        std::thread::park();
        
        Ok(())
    }
}

#[cfg(not(feature = "audio"))]
pub mod audio_io {
    //! Stub module when audio feature is not enabled

    /// Stub for play_to_speaker
    pub fn play_to_speaker(_data: &[u8]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Err("Audio feature not enabled. Rebuild with --features audio".into())
    }

    /// Stub for stream_to_speaker
    pub fn stream_to_speaker(
        _chunk_stream: impl futures_util::Stream<Item = Result<bytes::Bytes, Box<dyn std::error::Error + Send + Sync>>> + Send,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Err("Audio feature not enabled. Rebuild with --features audio".into())
    }

    /// Stub for record_from_microphone
    pub fn record_from_microphone(_duration_secs: f32) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        Err("Audio feature not enabled. Rebuild with --features audio".into())
    }

    /// Stub for stream_from_microphone
    pub fn stream_from_microphone(
        _chunk_duration_secs: f32,
        _callback: impl FnMut(&[u8]) + Send + 'static,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Err("Audio feature not enabled. Rebuild with --features audio".into())
    }
}
