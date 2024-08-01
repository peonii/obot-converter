use super::replay::{ReplayClickType, ReplayFormat};


pub struct PlainTextReplay {
    pub fps: f32,
    pub clicks: Vec<PlainTextClick>,
}

pub struct PlainTextClick {
    pub frame: i32,
    pub hold: bool,
    pub player_2: bool
}

impl PlainTextClick {
    pub fn write(&self) -> String {
        format!("{} {} {}", self.frame, if self.hold { 1 } else { 0 }, if self.player_2 { 1 } else { 0 })
    }

    pub fn from_string(data: &str) -> eyre::Result<Self> {
        let mut split = data.split_whitespace();
        let frame = split.next().ok_or(eyre::eyre!("No frame"))?.parse()?;
        let hold = split.next().ok_or(eyre::eyre!("No hold"))?.parse()?;
        let player_2 = split.next().ok_or(eyre::eyre!("No player 2"))?.parse()?;

        Ok(Self {
            frame,
            hold,
            player_2,
        })
    }
}

impl ReplayFormat for PlainTextReplay {
    type ClickType = PlainTextClick;

    fn new(fps: f32) -> Self {
        Self {
            fps,
            clicks: vec![],
        }
    }

    fn add_click(&mut self, click: Self::ClickType) -> () {
        self.clicks.push(click);
    }

    fn dump(&self) -> eyre::Result<Vec<u8>> {
        let mut data = String::new();
        data.push_str(&self.fps.to_string());
        data.push('\n');

        for click in &self.clicks {
            data.push_str(&click.write());
            data.push('\n');
        }

        Ok(data.into_bytes())
    }

    fn from_data(data: &mut std::io::Cursor<Vec<u8>>) -> eyre::Result<Self>
        where
            Self: Sized {
        let mut data = String::from_utf8(data.get_ref().clone())?;

        let mut split = data.split('\n');
        let fps = split.next().ok_or(eyre::eyre!("No fps"))?.parse()?;
        let mut clicks = vec![];

        for click in split {
            clicks.push(PlainTextClick::from_string(click)?);
        }

        Ok(Self {
            fps,
            clicks,
        })
    }

    fn from_universal(replay: super::replay::Replay) -> eyre::Result<Self>
        where
            Self: Sized {
        let mut plain_text_replay = PlainTextReplay::new(replay.fps);

        for click in replay.clicks.iter() {
            if click.p1 != ReplayClickType::Skip {
                plain_text_replay.add_click(PlainTextClick {
                    frame: click.frame as i32,
                    hold: click.p1 == ReplayClickType::Click,
                    player_2: false,
                });
            }

            if click.p2 != ReplayClickType::Skip {
                plain_text_replay.add_click(PlainTextClick {
                    frame: click.frame as i32,
                    hold: click.p2 == ReplayClickType::Click,
                    player_2: true,
                });
            }
        }

        Ok(plain_text_replay)
    }

    fn load(path: impl AsRef<std::path::Path>) -> eyre::Result<Self>
        where
            Self: Sized {
        let data = std::fs::read(path)?;

        let mut cursor = std::io::Cursor::new(data);
        Self::from_data(&mut cursor)
    }

    fn save(&self, path: impl AsRef<std::path::Path>) -> eyre::Result<()> {
        let data = self.dump()?;
        std::fs::write(path, data)?;
        Ok(())
    }

    fn to_universal(&self) -> eyre::Result<super::replay::Replay> {
        let mut replay = super::replay::Replay::new(self.fps, super::replay::GameVersion::Version2113);

        for click in self.clicks.iter() {
            if click.player_2 {
                replay.clicks.push(
                    super::replay::ReplayClick {
                        frame: click.frame as i64,
                        p1: ReplayClickType::Skip,
                        p2: if click.hold { ReplayClickType::Click } else { ReplayClickType::Release },
                    }
                )
            } else {
                replay.clicks.push(
                    super::replay::ReplayClick {
                        frame: click.frame as i64,
                        p1: if click.hold { ReplayClickType::Click } else { ReplayClickType::Release },
                        p2: ReplayClickType::Skip,
                    }
                )
            }
        }

        Ok(replay)
    }
}
