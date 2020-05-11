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

fn main() {
    let mut w: World = World::new_win(Vec2::new(640.0, 480.0), "Gomoku by Matyi", Vec3::grey(0.1));

    let rt = w.render_target().unwrap();

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
            saved_id: 0,
        },
    );
    gui.init_gl_res();
    gui.rebuild_gui();
    let id = w.add_actor(gui);
    w.make_actor_ui_aware(id);
    w.run();
}
