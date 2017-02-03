use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use messaging::{UpdateMessage, UpdateMessageSender};

use super::Mixer;
use super::AudioFilter;

pub struct SoftMixer {
  volume: Arc<AtomicUsize>,
  tx: Option<UpdateMessageSender>
}

impl SoftMixer {
    pub fn new() -> SoftMixer {
        SoftMixer {
            volume: Arc::new(AtomicUsize::new(0xFFFF)),
            tx: None
        }
    }
}

impl Mixer for SoftMixer {
    fn init(&mut self, tx: UpdateMessageSender) {
        self.tx = Some(tx);
    }
    fn start(&self) {
    }
    fn stop(&self) {
    }
    fn volume(&self) -> u16 {
        self.volume.load(Ordering::Relaxed) as u16
    }
    fn set_volume(&self, volume: u16) {
        self.volume.store(volume as usize, Ordering::Relaxed);
        let tx = self.tx.as_ref().expect("SoftMixer not initialized");
        tx.send(UpdateMessage).unwrap();
    }
    fn get_audio_filter(&self) -> Option<Box<AudioFilter + Send>> {
        Some(Box::new(SoftVolumeApplier { volume: self.volume.clone() }))
    }
}

struct SoftVolumeApplier {
  volume: Arc<AtomicUsize>
}

impl AudioFilter for SoftVolumeApplier {
    fn modify_stream(&self, data: &mut [i16]) {
        let volume = self.volume.load(Ordering::Relaxed) as u16;
        if volume != 0xFFFF {
            let factor = volume as i32 / 0xFFFF;
            for x in data.iter_mut() {
                *x = (*x as i32 * factor) as i16;
            }
        }
    }
}