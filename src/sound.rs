use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use sdl2::audio::{AudioCallback, AudioCVT, AudioDevice, AudioSpecDesired, AudioSpecWAV};
use sdl2::Sdl;

struct Sound {
    data: Vec<u8>,
    volume: f32,
    pos: usize,
    end: usize,
    counter: i32,
    id: usize,
    repeat:bool,
}

pub enum Playing {
    Playing,
    Ended,
    Finished,
}

impl AudioCallback for Sound {
    type Channel = u8;

    fn callback(&mut self, out: &mut [u8]) {
        for dst in out.iter_mut() {
            let pre_scale = *self.data.get(self.pos).unwrap_or(&128);
            let scaled_signed_float = (pre_scale as f32 - 128.0) * self.volume;
            let scaled = (scaled_signed_float + 128.0) as u8;
            *dst = scaled;

            unsafe {
                let currently = PLAYING_MAP.as_mut().unwrap().get(&self.id);
                let volume = VOLUME_MAP.as_mut().unwrap().get(&self.id).unwrap();
                match currently {
                    Some(Playing::Playing) => {
                        self.pos += 1;
                        self.volume = *volume;
                        self.counter = self.counter + 1;
                        if self.pos >= self.end {
                            if ! self.repeat {
                                PLAYING_MAP.as_mut().unwrap().insert(self.id, Playing::Ended);
                            }
                            self.pos = 0;
                        }
                    },
                    _ => {

                    },
                }
            }
        }
    }
}

static mut VOLUME_MAP: Option<HashMap<usize, f32>> = None;
static mut PLAYING_MAP: Option<HashMap<usize, Playing>> = None;
static mut SOUNDS_MAP: Option<HashMap<usize, AudioDevice<Sound>>> = None;

pub static HIT_WALL: usize = 1;
pub static ENGINE: usize = 2;
pub static FIRE: usize = 3;

#[cfg(feature = "soundoff")]
pub fn load_sound(sdl_context: &Sdl) { }
#[cfg(feature = "soundoff")]
pub fn play(id: usize) { }
#[cfg(feature = "soundoff")]
pub fn _pause_any_finished_sounds() { }
#[cfg(feature = "soundoff")]
pub fn _stop(id: usize) { }
#[cfg(feature = "soundoff")]
pub fn _volume(id:usize,volume:f32) {}

#[cfg(not(feature = "soundoff"))]
pub fn load_sound(sdl_context: &Sdl) {
    unsafe {
        PLAYING_MAP = Some(HashMap::new());
        SOUNDS_MAP = Some(HashMap::new());
        VOLUME_MAP = Some(HashMap::new());
        SOUNDS_MAP.as_mut().unwrap().insert(HIT_WALL, load_in_file(sdl_context, "resources/sound/hit.wav", HIT_WALL,  false));
        SOUNDS_MAP.as_mut().unwrap().insert(ENGINE, load_in_file(sdl_context, "resources/sound/242740__marlonhj__engine.wav", ENGINE, true));
        SOUNDS_MAP.as_mut().unwrap().insert(FIRE, load_in_file(sdl_context, "resources/sound/fire.wav", FIRE, false));
    }
}

#[cfg(not(feature = "soundoff"))]
pub fn play(id: usize) {
    unsafe {
        PLAYING_MAP.as_mut().unwrap().insert(id, Playing::Playing);
        SOUNDS_MAP.as_mut().unwrap().get(&id).as_ref().unwrap().resume();
    }
}

#[cfg(not(feature = "soundoff"))]
pub fn _stop(id: usize) {
    unsafe {
        PLAYING_MAP.as_mut().unwrap().insert(id, Playing::Finished);
        SOUNDS_MAP.as_mut().unwrap().get(&id).as_ref().unwrap().pause();
    }
}

#[cfg(not(feature = "soundoff"))]
pub fn _volume(id:usize,volume:f32) {
    unsafe {
        VOLUME_MAP.as_mut().unwrap().insert(id, volume);
    }

}

#[cfg(not(feature = "soundoff"))]
pub fn _pause_any_finished_sounds() {
    unsafe {
        for (k, v) in PLAYING_MAP.as_ref().unwrap().iter() {
            match v {
                Playing::Ended => {
                    SOUNDS_MAP.as_mut().unwrap().get(k).as_ref().unwrap().pause();
                    PLAYING_MAP.as_mut().unwrap().insert(*k, Playing::Finished);
                }
                _ => {}
            }
        }
    }
}

fn load_in_file(sdl_context: &Sdl, file_name: &'static str, id: usize, repeat:bool) -> AudioDevice<Sound> {
    unsafe {
        match &mut PLAYING_MAP {
            Some(p) => {
                p.insert(id, Playing::Ended);
            }
            None => println!("WHAT!!!"),
        }
        match &mut VOLUME_MAP {
            Some(p) => {
                p.insert(id,15.0);
            }
            None => println!("What??"),
        }
    }

    let wav_file: Cow<'static, Path> = match std::env::args().nth(1) {
        None => Cow::from(Path::new(file_name)),
        Some(s) => Cow::from(PathBuf::from(s))
    };
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(1), // mono
        samples: None,      // default
    };

    let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        let wav = AudioSpecWAV::load_wav(wav_file)
            .expect("Could not load test WAV file");

        let cvt = AudioCVT::new(
            wav.format, wav.channels, wav.freq,
            spec.format, spec.channels, spec.freq)
            .expect("Could not convert WAV file");

        let data = cvt.convert(wav.buffer().to_vec());

        let size = data.len() ;
        // initialize the audio callback
        Sound {
            data,
            volume: 0.3,
            pos: 0,
            end: size,
            counter: 0,
            id:id,
            repeat:repeat,
        }
    }).unwrap();
    device
}

