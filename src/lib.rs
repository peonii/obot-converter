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
    MHRJson,
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
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn console_error(s: &str);
}

#[wasm_bindgen]
impl Converter {
    pub fn load(&mut self, data: Vec<u8>, fmt: Format) -> Result<(), ConverterError> {
        let cursor = Cursor::new(data);

        self.loaded_replay.clear();

        let result = match fmt {
            Format::PlainText => self.loaded_replay.parse_plain_text(cursor),
            Format::Tasbot => self.loaded_replay.parse_tasbot(cursor),
            Format::ZBot => self.loaded_replay.parse_zbot(cursor),
            Format::OmegaBot => self.loaded_replay.parse_obot3(cursor),
            Format::OmegaBot2 => self.loaded_replay.parse_obot2(cursor),
            Format::URL => self.loaded_replay.parse_url(cursor),
            Format::MHRJson => self.loaded_replay.parse_mhr_json(cursor),
            Format::MHRBinary => self.loaded_replay.parse_mhr_binary(cursor),
        };

        match result {
            Ok(_) => {},
            Err(e) => {
                console_error(&e.to_string());
            }
        }

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

        let result = match fmt {
            Format::PlainText => self.loaded_replay.write_plain_text(&mut cursor),
            Format::Tasbot => self.loaded_replay.write_tasbot(&mut cursor),
            Format::ZBot => self.loaded_replay.write_zbot(&mut cursor),
            Format::OmegaBot => self.loaded_replay.write_obot3(&mut cursor),
            Format::OmegaBot2 => self.loaded_replay.write_obot2(&mut cursor),
            Format::URL => self.loaded_replay.write_url(&mut cursor),
            Format::MHRJson => self.loaded_replay.write_mhr_json(&mut cursor),
            Format::MHRBinary => self.loaded_replay.write_mhr_binary(&mut cursor),
        };

        match result {
            Ok(_) => {},
            Err(e) => {
                console_error(&e.to_string());
            }
        }

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