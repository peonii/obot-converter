pub mod formats;

use std::io::Cursor;

use formats::replay::{Click, ClickType, GameVersion, Replay};
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
    OmegaBot2,
    URL,
    MHRJson,
    MHRBinary,
    Tasbot,
    ZBot,
    ReplayBot,
    Fembot,
    EchoOld,
    EchoNewJson,
    EchoNewBinary,
    YBot1,
    XBot,
    Rush,
    KDBot,
    YBot,
    GDR,
    GDRJson,
    XDBot,
    PlainText,
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

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn console_log(s: &str);
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

    pub fn set_fps(&mut self, fps: f32) {
        self.loaded_replay.fps = fps;
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

    pub fn clicks(&self) -> Vec<Click> {
        self.loaded_replay.clicks.clone()
    }

    pub fn click_at(&self, idx: usize) -> Click {
        self.loaded_replay.clicks[idx]
    }

    pub fn clicks_at_batch(&self, idx: usize, page: usize) -> Vec<Click> {
        self.loaded_replay.clicks[idx..idx+page].to_vec()
    }

    pub fn replace_frame_at(&mut self, idx: usize, frame: u32) {
        self.loaded_replay.clicks[idx].frame = frame;
    }

    pub fn insert_empty_at(&mut self, idx: usize, frame: u32) {
        self.loaded_replay.clicks.insert(idx, Click {
            frame,
            p1: ClickType::Skip,
            p2: ClickType::Skip
        })
    }

    pub fn remove_at(&mut self, idx: usize) {
        self.loaded_replay.clicks.remove(idx);
    }

    pub fn toggle_click_at(&mut self, idx: usize, player_2: bool) {
        if player_2 {
            self.loaded_replay.clicks[idx].p2 = self.loaded_replay.clicks[idx].p2.toggle();
        } else {
            self.loaded_replay.clicks[idx].p1 = self.loaded_replay.clicks[idx].p1.toggle();
        }
    }

    pub fn clean(&mut self) {
        let mut current_click_state_p1 = false;
        let mut current_click_state_p2 = false;

        let clicks_old = self.loaded_replay.clicks.clone();
        self.loaded_replay.clicks = clicks_old.into_iter().map(|click| {
            let mut new_click = click;
            
            if click.p1.is_click() {
                let valid = if !current_click_state_p1 { true } else { false };
                current_click_state_p1 = if valid { true } else { current_click_state_p1 };
                if !valid {
                    console_log(&format!("CLEANED {} - turned reduntant p1 click into skip", click.frame));
                    new_click.p1 = ClickType::Skip;   
                }
            } else if click.p1.is_release() {
                let valid = if current_click_state_p1 { true } else { false };
                current_click_state_p1 = if valid { false } else { current_click_state_p1 };
                if !valid {
                    console_log(&format!("CLEANED {} - turned reduntant p1 release into skip", click.frame));
                    new_click.p1 = ClickType::Skip;   
                }
            }

            if click.p2.is_click() {
                let valid = if !current_click_state_p2 { true } else { false };
                current_click_state_p2 = if valid { true } else { current_click_state_p2 };
                if !valid {
                    console_log(&format!("CLEANED {} - turned reduntant p2 click into skip", click.frame));
                    new_click.p2 = ClickType::Skip;   
                }
            } else if click.p2.is_release() {
                let valid = if current_click_state_p2 { true } else { false };
                current_click_state_p2 = if valid { false } else { current_click_state_p2 };
                if !valid {
                    console_log(&format!("CLEANED {} - turned reduntant p2 release into skip", click.frame));
                    new_click.p2 = ClickType::Skip;   
                }
            }

            return new_click;
        })
        .filter(|click| {
            if click.p1.is_skip() && click.p2.is_skip() {
                console_log(&format!("CLEANED {} - removed input as both p1 and p2 were skips", click.frame));
                return false;
            }

            return true;
        }).collect();

        console_log("Successfully cleaned replay");
    }

    pub fn sort(&mut self) {
        self.loaded_replay.clicks.sort_unstable_by(|c1, c2| {
            c1.frame.cmp(&c2.frame)
        });

        console_log("Successfully sorted inputs");
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
    console_error_panic_hook::set_once();

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let _body = document.body().expect("document should have a body");
}