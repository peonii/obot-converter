pub mod cli;
pub mod formats;

use std::io::Cursor;

use formats::{mhr::MHRReplay, mhr_binary::MHRBinaryReplay, omegabot::OmegabotReplay, plain_text::PlainTextReplay, replay::{GameVersion, Replay, ReplayClick, ReplayFormat}, tasbot::TasbotReplay, zbot::ZBotReplay};
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
#[derive(Clone)]
pub struct Converter {
    loaded_replay: Replay
}

#[wasm_bindgen]
pub enum Format {
    OmegaBot,
    MHR,
    Tasbot,
    MHRBinary,
    ZBot,
    PlainText
}

#[wasm_bindgen]
pub enum ConverterError {
    InvalidData
}

#[wasm_bindgen]
impl Converter {
    pub fn load(&mut self, data: Vec<u8>, fmt: Format) -> Result<(), ConverterError> {
        let mut data = Cursor::new(data);

        self.loaded_replay = match fmt {
            Format::OmegaBot => {
                OmegabotReplay::from_data(&mut data).map_err(|_| ConverterError::InvalidData)?.to_universal().map_err(|_| ConverterError::InvalidData)?
            }
            Format::MHR => {
                MHRReplay::from_data(&mut data).map_err(|_| ConverterError::InvalidData)?.to_universal().map_err(|_| ConverterError::InvalidData)?
            }
            Format::Tasbot => {
                TasbotReplay::from_data(&mut data).map_err(|_| ConverterError::InvalidData)?.to_universal().map_err(|_| ConverterError::InvalidData)?
            }
            Format::MHRBinary => {
                MHRBinaryReplay::from_data(&mut data).map_err(|_| ConverterError::InvalidData)?.to_universal().map_err(|_| ConverterError::InvalidData)?
            }
            Format::ZBot => {
                ZBotReplay::from_data(&mut data).map_err(|_| ConverterError::InvalidData)?.to_universal().map_err(|_| ConverterError::InvalidData)?
            }
            Format::PlainText => {
                PlainTextReplay::from_data(&mut data).map_err(|_| ConverterError::InvalidData)?.to_universal().map_err(|_| ConverterError::InvalidData)?
            }
        };

        Ok(())
    }

    pub fn get_fps(&self) -> f32 {
        self.loaded_replay.fps
    }

    pub fn length(&self) -> usize {
        self.loaded_replay.clicks.len()
    }

    pub fn clicks(&self) -> Vec<ReplayClick> {
        self.loaded_replay.clicks.clone()
    }

    pub fn save(&self, fmt: Format) -> Vec<u8> {
        match fmt {
            Format::OmegaBot => {
                let replay = OmegabotReplay::from_universal(self.loaded_replay.clone()).unwrap();
                replay.dump().unwrap()
            }
            Format::MHR => {
                let replay = MHRReplay::from_universal(self.loaded_replay.clone()).unwrap();
                replay.dump().unwrap()
            }
            Format::Tasbot => {
                let replay = TasbotReplay::from_universal(self.loaded_replay.clone()).unwrap();
                replay.dump().unwrap()
            }
            Format::MHRBinary => {
                let replay = MHRBinaryReplay::from_universal(self.loaded_replay.clone()).unwrap();
                replay.dump().unwrap()
            }
            Format::ZBot => {
                let replay = ZBotReplay::from_universal(self.loaded_replay.clone()).unwrap();
                replay.dump().unwrap()
            }
            Format::PlainText => {
                let replay = PlainTextReplay::from_universal(self.loaded_replay.clone()).unwrap();
                replay.dump().unwrap()
            }
        }
    }

    #[wasm_bindgen(constructor)]
    pub fn new() -> Converter {
        Converter {
            loaded_replay: Replay::new(60.0, GameVersion::Version2113)
        }
    }
}

#[wasm_bindgen(start)]
fn run() {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");
}