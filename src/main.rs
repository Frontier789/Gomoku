#![windows_subsystem = "windows"]
extern crate glui;
extern crate glui_proc;
extern crate rand;

use glui::gui::*;
use glui::mecs::*;
use glui::tools::*;

mod ai;
mod board;
mod gamestate;
mod ui;
use board::*;
use gamestate::*;
use ui::*;

use std::fs::OpenOptions;
use std::io::Write;

// use ai::ai_eval;
// use std::fs::File;
// use std::io;
// use std::io::BufRead;

fn main() {
    // let mut lines: Vec<String> = Default::default();
    //
    // if let Ok(file) = File::open("data/games.txt") {
    //     lines = io::BufReader::new(file)
    //         .lines()
    //         .map(|r| r.unwrap_or_default())
    //         .filter(|l| l.len() > 0 && l.as_bytes()[0] == '[' as u8)
    //         .collect();
    // }
    //
    // println!("{} entries read", lines.len());
    //
    // let mut file = OpenOptions::new()
    //     .write(true)
    //     .create(true)
    //     .open("data/evals.txt")
    //     .unwrap();
    // for i in 0..lines.len() {
    //     let s = &lines[i];
    //     let vals = ai_eval(s, true);
    //     writeln!(file, "{}\n{:?}", s, vals.unwrap()).unwrap();
    //     println!("Entry {} done!", i + 1);
    // }

    let mut w: World = World::new_win(Vec2::new(640.0, 480.0), "Gomoku by Matyi", Vec3::grey(0.1));

    let rt = w.window_info().unwrap();

    if let Ok(mut file) = OpenOptions::new().create(true).open("ogl.txt") {
        let _ = write!(file, "OpenGL version: {}", rt.gl_verison);
    };

    let mut gui = GuiContext::new(
        rt,
        true,
        GameData {
            board: Default::default(),
            state: GameState::MainMenu,
            intelligence: (PlayerInt::Human, PlayerInt::Human),
            save_id: 0,
        },
    );
    gui.rebuild_gui();
    let id = w.add_system(gui);
    w.make_system_ui_aware(id);
    w.run();
}
