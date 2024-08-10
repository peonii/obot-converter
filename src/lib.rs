pub mod formats;

use std::io::Cursor;

use formats::replay::{GameVersion, Replay};
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
    OmegaBot2,
    YBot,
    GDR,
    GDRJson,
    ReplayBot,
    Fembot,
    YBot1,
    EchoOld,
    EchoNewJson,
    EchoNewBinary,
    XBot,
    Rush,
    KDBot,
    XDBot
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
            Format::YBot => self.loaded_replay.parse_ybot2(cursor),
            Format::GDR => self.loaded_replay.parse_gdr(cursor),
            Format::GDRJson => self.loaded_replay.parse_gdr_json(cursor),
            Format::ReplayBot => self.loaded_replay.parse_replaybot(cursor),
            Format::YBot1 => self.loaded_replay.parse_ybot1(cursor),
            Format::EchoOld => self.loaded_replay.parse_echo_old(cursor),
            Format::EchoNewJson => self.loaded_replay.parse_echo_new(cursor),
            Format::EchoNewBinary => self.loaded_replay.parse_echo_bin(cursor),
            Format::Rush => self.loaded_replay.parse_rush(cursor),
            Format::KDBot => self.loaded_replay.parse_kdbot(cursor),
            Format::XBot => self.loaded_replay.parse_xbot(cursor),
            Format::XDBot => self.loaded_replay.parse_xdbot(cursor),
            Format::Fembot => self.loaded_replay.parse_fembot(cursor)
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

    // pub fn clicks(&self) -> Vec<Click> {
    //     self.loaded_replay.clicks.clone()
    // }

    pub fn game_version(&self) -> GameVersion {
        self.loaded_replay.game_version
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
            Format::YBot => self.loaded_replay.write_ybot2(&mut cursor),
            Format::GDR => self.loaded_replay.write_gdr(&mut cursor),
            Format::GDRJson => self.loaded_replay.write_gdr_json(&mut cursor),
            Format::ReplayBot => self.loaded_replay.write_replaybot(&mut cursor),
            Format::YBot1 => self.loaded_replay.write_ybot1(&mut cursor),
            Format::EchoOld => self.loaded_replay.write_echo_old(&mut cursor),
            Format::EchoNewJson => self.loaded_replay.write_echo_new(&mut cursor),
            Format::EchoNewBinary => self.loaded_replay.write_echo_bin(&mut cursor),
            Format::Rush => self.loaded_replay.write_rush(&mut cursor),
            Format::KDBot => self.loaded_replay.write_kdbot(&mut cursor),
            Format::XBot => self.loaded_replay.write_xbot(&mut cursor),
            Format::XDBot => self.loaded_replay.write_xdbot(&mut cursor),
            Format::Fembot => self.loaded_replay.write_fembot(&mut cursor)
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