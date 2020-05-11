extern crate glui;
extern crate glui_proc;
extern crate rand;

use board::GameResult;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PlayerInt {
    Human,
    AI,
}

impl PlayerInt {
    pub fn to_string(self) -> String {
        match self {
            PlayerInt::Human => "Human".to_owned(),
            PlayerInt::AI => "AI".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GameState {
    MainMenu,
    Playing,
    LoadSaved,
    Finished(GameResult),
}
