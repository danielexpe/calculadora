use std::sync::mpsc::SyncSender;
use std::thread::JoinHandle;

#[cfg(feature = "audio")]
use std::sync::mpsc;
#[cfg(feature = "audio")]
use std::thread;
#[cfg(feature = "audio")]
use std::time::Duration;

pub const SAMPLE_RATE: u32 = 48_000;
pub const VOLUME: f32 = 0.32;
pub const CLICK_MS: u32 = 22;
pub const ERROR_MS: u32 = 180;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundEvent {
    Click,
    Error,
}

impl SoundEvent {
    pub const fn duration_ms(self) -> u32 {
        match self {
            Self::Click => CLICK_MS,
            Self::Error => ERROR_MS,
        }
    }
}

pub fn samples(event: SoundEvent) -> Vec<f32> {
    match event {
        SoundEvent::Click => click_samples(),
        SoundEvent::Error => error_samples(),
    }
}

pub fn pcm16(event: SoundEvent) -> Vec<i16> {
    samples(event)
        .into_iter()
        .map(|sample| (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16)
        .collect()
}

fn click_samples() -> Vec<f32> {
    let frames = frames(CLICK_MS);
    let mut samples = Vec::with_capacity(frames);
    let mut state = 0x1234_5678u32;
    let mut previous = 0.0;

    for index in 0..frames {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let noise = ((state >> 8) as f32 / 0x00ff_ffff as f32) * 2.0 - 1.0;
        let filtered = noise * 0.65 + previous * 0.35;
        previous = filtered;
        let progress = index as f32 / frames as f32;
        let attack = (progress / 0.05).min(1.0);
        let envelope = attack * (1.0 - progress).powi(3);
        samples.push(filtered * envelope * VOLUME * 0.8);
    }

    samples
}

fn error_samples() -> Vec<f32> {
    let frames = frames(ERROR_MS);
    let mut samples = Vec::with_capacity(frames);
    let frequency = 140.0;

    for index in 0..frames {
        let time = index as f32 / SAMPLE_RATE as f32;
        let progress = index as f32 / frames as f32;
        let attack = (progress / 0.03).min(1.0);
        let envelope = attack * (1.0 - progress).powi(2);
        let wave = (std::f32::consts::TAU * frequency * time).sin() * 0.7
            + (std::f32::consts::TAU * frequency * 2.0 * time).sin() * 0.2;
        samples.push(wave * envelope * VOLUME);
    }

    samples
}

fn frames(milliseconds: u32) -> usize {
    (SAMPLE_RATE as u64 * milliseconds as u64 / 1000) as usize
}

pub struct Audio {
    sender: Option<SyncSender<SoundEvent>>,
    worker: Option<JoinHandle<()>>,
    available: bool,
}

impl Audio {
    pub fn start() -> Self {
        #[cfg(feature = "audio")]
        {
            let (sender, receiver) = mpsc::sync_channel(8);
            let (ready_sender, ready_receiver) = mpsc::channel();
            let worker = thread::spawn(move || {
                crate::platform::audio::run(receiver, ready_sender);
            });
            let available = ready_receiver
                .recv_timeout(Duration::from_millis(250))
                .unwrap_or(false);
            if available {
                return Self {
                    sender: Some(sender),
                    worker: Some(worker),
                    available: true,
                };
            }
            drop(sender);
            let _ = worker.join();
            Self::unavailable()
        }

        #[cfg(not(feature = "audio"))]
        {
            Self::unavailable()
        }
    }

    pub fn unavailable() -> Self {
        Self {
            sender: None,
            worker: None,
            available: false,
        }
    }

    pub const fn is_available(&self) -> bool {
        self.available
    }

    pub fn play(&self, event: SoundEvent) -> bool {
        self.sender
            .as_ref()
            .is_some_and(|sender| sender.try_send(event).is_ok())
    }
}

impl Drop for Audio {
    fn drop(&mut self) {
        self.sender = None;
        if let Some(worker) = self.worker.take() {
            let deadline = std::time::Instant::now() + std::time::Duration::from_millis(300);
            while !worker.is_finished() && std::time::Instant::now() < deadline {
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            if worker.is_finished() {
                let _ = worker.join();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn durations_match_the_specification() {
        assert_eq!(frames(CLICK_MS), 1_056);
        assert_eq!(frames(ERROR_MS), 8_640);
        assert_eq!(samples(SoundEvent::Click).len(), 1_056);
        assert_eq!(samples(SoundEvent::Error).len(), 8_640);
        assert_eq!(SoundEvent::Click.duration_ms(), CLICK_MS);
        assert_eq!(SoundEvent::Error.duration_ms(), ERROR_MS);
    }

    #[test]
    fn synthesis_is_finite_and_bounded() {
        for event in [SoundEvent::Click, SoundEvent::Error] {
            let samples = samples(event);
            assert!(samples.iter().all(|sample| sample.is_finite()));
            assert!(samples.iter().all(|sample| sample.abs() <= 1.0));
            assert!(samples.iter().any(|sample| sample.abs() > 0.01));
            assert!(samples.first().copied().unwrap().abs() < 1e-6);
            assert!(samples.last().copied().unwrap().abs() < 1e-3);
        }
    }

    #[test]
    fn pcm_conversion_stays_in_range_and_is_deterministic() {
        for event in [SoundEvent::Click, SoundEvent::Error] {
            let first = pcm16(event);
            let second = pcm16(event);
            assert_eq!(first, second);
            assert_eq!(first.len(), samples(event).len());
        }
    }

    #[test]
    fn unavailable_backend_is_silent_and_non_blocking() {
        let audio = Audio::unavailable();
        assert!(!audio.is_available());
        assert!(!audio.play(SoundEvent::Click));
    }

    #[test]
    fn backend_start_is_bounded_and_optional() {
        let started = Instant::now();
        let audio = Audio::start();
        assert!(started.elapsed() < Duration::from_millis(400));
        if !audio.is_available() {
            assert!(!audio.play(SoundEvent::Click));
        }
    }
}
