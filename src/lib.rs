#![feature(seek_stream_len)]
pub mod formats;

use std::io::Cursor;

use formats::replay::{Click, Replay};
use thiserror::Error;
use wasm_bindgen::prelude::*;

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
    PlainText,
    URL,
    OmegaBot2
}

#[wasm_bindgen]
#[derive(Debug, Error)]
pub enum ConverterError {
    #[error("Invalid data provided")]
    InvalidData,
}

#[wasm_bindgen]
impl Converter {
    pub fn load(&mut self, data: Vec<u8>, fmt: Format) -> Result<(), ConverterError> {
        let cursor = Cursor::new(data);

        self.loaded_replay.clear();

        match fmt {
            Format::PlainText => {
                self.loaded_replay.parse_plain_text(cursor).unwrap()     
            }
            Format::Tasbot => {
                self.loaded_replay.parse_tasbot(cursor).unwrap()
            }
            Format::ZBot => {
                self.loaded_replay.parse_zbot(cursor).unwrap()
            }
            Format::OmegaBot => {
                self.loaded_replay.parse_obot3(cursor).unwrap()
            }
            Format::MHR => {
                self.loaded_replay.parse_mhr_json(cursor).unwrap()
            }
            Format::MHRBinary => {
                self.loaded_replay.parse_mhr_binary(cursor).unwrap()
            }
            Format::URL => {
                self.loaded_replay.parse_url(cursor).unwrap()
            }
            Format::OmegaBot2 => {
                self.loaded_replay.parse_obot2(cursor).unwrap()
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

    pub fn clicks(&self) -> Vec<Click> {
        self.loaded_replay.clicks.clone()
    }

    pub fn save(&self, fmt: Format) -> Vec<u8> {
        let buffer = Vec::new();
        let mut cursor = Cursor::new(buffer);

        match fmt {
            Format::PlainText => {
                self.loaded_replay.write_plain_text(&mut cursor)
                    .unwrap();
            }
            Format::Tasbot => {
                self.loaded_replay.write_tasbot(&mut cursor)
                    .unwrap()
            }
            Format::ZBot => {
                self.loaded_replay.write_zbot(&mut cursor)
                    .unwrap()
            }
            Format::OmegaBot => {
                self.loaded_replay.write_obot3(&mut cursor)
                    .unwrap()
            }
            Format::MHR => {
                self.loaded_replay.write_mhr_json(&mut cursor)
                    .unwrap()
            }
            Format::MHRBinary => {
                self.loaded_replay.write_mhr_binary(&mut cursor)
                    .unwrap()
            }
            Format::URL => {
                self.loaded_replay.write_url(&mut cursor)
                    .unwrap()
            }
            Format::OmegaBot2 => {
                self.loaded_replay.write_obot2(&mut cursor)
                    .unwrap()
            }
        };

        cursor.into_inner()
    }

    #[wasm_bindgen(constructor)]
    pub fn new() -> Converter {
        Converter {
            loaded_replay: Replay::default()
        }
    }
}

#[wasm_bindgen(start)]
fn run() {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let _body = document.body().expect("document should have a body");
}