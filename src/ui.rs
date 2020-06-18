extern crate glui;
extern crate glui_proc;
extern crate rand;

use std::ops::Neg;
use std::ops::Shl;

use super::ai::*;
use super::board::*;
use super::gamestate::{GameState, PlayerInt};
use glui::gui::*;
use glui::impl_widget_building_for;
use glui::mecs::*;
use glui::tools::*;
use std::fs::File;
use std::io;
use std::io::BufRead;

#[derive(Clone, Debug, PartialEq)]
pub struct GameData {
    pub board: Board,
    pub state: GameState,
    pub intelligence: (PlayerInt, PlayerInt),
    pub save_id: usize,
}

impl GuiBuilder for GameData {
    fn build(&self) {
        match self.state.clone() {
            GameState::MainMenu => {
                self.main_menu_gui();
            }
            GameState::Playing => {
                self.game_gui(GameResult::NotFinished);
            }
            GameState::LoadSaved => {
                self.load_saved_gui(self.save_id);
            }
            GameState::Finished(r) => {
                self.game_gui(r);
            }
        }
    }
}

#[allow(unused_must_use)]
impl GameData {
    fn load_saved_gui(&self, mut id: usize) {
        -GridLayout {
            row_heights: GuiDimension::relative_array(vec![0.9, 0.1]),
            ..Default::default()
        } << {
            let mut lines: Vec<String> = Default::default();

            if let Ok(file) = File::open("data/games.txt") {
                lines = io::BufReader::new(file)
                    .lines()
                    .map(|r| r.unwrap_or_default())
                    .collect();
            }

            if !lines.is_empty() {
                let mut board = Board::default();
                if 2 * id + 1 >= lines.len() {
                    id = 0;
                }

                board.load_from_string(&lines[2 * id + 1]);
                -GridLayout {
                    col_widths: GuiDimension::relative_array(vec![0.2, 1.0, 0.2]),
                    ..Default::default()
                } << {
                    -Padding::ratios(0.0, 0.1, 0.3, 0.2)
                        << if id > 0 {
                            -Button {
                                background: ButtonBckg::Image(
                                    "images/left".to_owned(),
                                    Vec4::WHITE.with_w(0.3),
                                    Vec4::WHITE.with_w(0.4),
                                    Vec4::WHITE.with_w(0.25),
                                ),
                                callback: self.make_callback1(|data| data.save_id -= 1),
                                ..Default::default()
                            };
                        };
                    -GridLayout {
                        row_heights: GuiDimension::relative_array(vec![0.1, 1.0]),
                        ..Default::default()
                    } << {
                        -Text {
                            text: lines[2 * id].clone(),
                            color: Vec4::WHITE,
                            ..Default::default()
                        };
                        -Square::default() << self.board_gui(board.clone(), GameResult::Draw);
                    };
                    -Padding::ratios(0.1, 0.0, 0.3, 0.2)
                        << if (id + 1) * 2 < lines.len() {
                            -Button {
                                background: ButtonBckg::Image(
                                    "images/right".to_owned(),
                                    Vec4::WHITE.with_w(0.3),
                                    Vec4::WHITE.with_w(0.4),
                                    Vec4::WHITE.with_w(0.2),
                                ),
                                callback: self.make_callback1(|data| data.save_id += 1),
                                ..Default::default()
                            };
                        };
                };
                -GridLayout {
                    col_widths: GuiDimension::relative_array(vec![0.5, 0.5]),
                    ..Default::default()
                } << {
                    self.button(
                        "Main menu",
                        self.make_callback1(|data| {
                            data.state = GameState::MainMenu;
                        }),
                        0.5,
                    );
                    self.button(
                        "Play",
                        self.make_callback1(move |data| {
                            data.state = GameState::Playing;
                            data.board = board.clone();
                            ai_new_game(data.board.moves());
                        }),
                        0.5,
                    );
                }
            } else {
                -Text {
                    text: "No games found".to_owned(),
                    color: Vec4::WHITE,
                    ..Default::default()
                };
                self.button(
                    "Main menu",
                    self.make_callback1(|data| {
                        data.state = GameState::MainMenu;
                    }),
                    0.5,
                );
            }
        }
    }
    fn human_comes(&self) -> bool {
        self.board
            .human_comes(self.intelligence.0, self.intelligence.1)
    }
    fn game_gui(&self, result: GameResult) {
        -OuterImage {
            name: "images/wood".to_owned(),
            mid: Vec2::new(0.5, 0.5),
            ..Default::default()
        } << -FixedPanel {
            dir: PanelDirection::Right,
            size: GuiDimension::Units(160.0),
            ..Default::default()
        } << {
            self.sidebar_gui(self.board.can_undo(), self.board.can_redo(), result.clone());
            self.board_gui(self.board.clone(), result);
        };
    }

    fn sidebar_gui(&self, can_undo: bool, can_redo: bool, result: GameResult) {
        let mut title = "Playing".to_owned();

        match result {
            GameResult::NotFinished => (),
            _ => {
                title = result.win_text();
            }
        };

        -GridLayout {
            row_heights: GuiDimension::relative_array(vec![
                0.1, 0.15, 0.15, 0.15, 0.4, 0.15, 0.15, 0.15,
            ]),
            ..Default::default()
        } << {
            -Overlay::from(Vec4::WHITE.with_w(0.6))
                << -Text {
                    text: title,
                    ..Default::default()
                };
            if can_undo {
                self.button(
                    "undo",
                    self.make_callback1(|data| {
                        data.board.undo();
                        ai_undo();
                        data.state = GameState::Playing;
                    }),
                    0.5,
                );
                self.button(
                    "undo all",
                    self.make_callback1(|data| {
                        for _ in 0..data.board.moves().len() {
                            data.board.undo();
                            ai_undo();
                        }
                        data.state = GameState::Playing;
                    }),
                    0.5,
                );
            } else {
                -Padding::default();
                -Padding::default();
            }

            if can_redo {
                self.button(
                    "redo",
                    self.make_callback1(|data| {
                        if let Some(p) = data.board.redo_step() {
                            let r = data.board.redo();
                            if r != GameResult::NotFinished {
                                data.state = GameState::Finished(r);
                            }
                            ai_redo(p);
                        }
                    }),
                    0.5,
                );
            } else {
                -Padding::default();
            }
            -Padding::default();
            if result.over() {
                self.button(
                    "New Game",
                    self.make_callback1(|data| {
                        data.state = GameState::Playing;
                        data.board = Board::default();
                        if data.intelligence.0 == PlayerInt::AI
                            || data.intelligence.1 == PlayerInt::AI
                        {
                            ai_new_game(data.board.moves());
                            if !data.human_comes() {
                                ai_move(&mut data.board);
                            }
                        }
                    }),
                    0.5,
                );
            } else {
                -Padding::default();
            }
            self.button(
                "Save",
                self.make_callback1(|data| {
                    data.board.append_game_to_saves();
                }),
                0.5,
            );
            self.button(
                "Main menu",
                self.make_callback1(|data| {
                    data.state = GameState::MainMenu;
                    data.board = Board::default();
                }),
                0.5,
            );
        }
    }

    fn int_text(&self) -> String {
        if self.intelligence.0 == self.intelligence.1 {
            "Humans".to_owned()
        } else {
            self.intelligence.0.to_string() + " vs " + &self.intelligence.1.to_string()
        }
    }

    fn main_menu_gui(&self) {
        -OuterImage {
            name: "images/main_menu".to_owned(),
            mid: Vec2::new_xy(0.5),
            ..Default::default()
        };
        -Square::default()
            << -Padding::relative_x(1.0 / 5.0)
            << -Overlay::from(Vec4::WHITE.with_w(0.2))
            << -GridLayout {
                row_heights: GuiDimension::relative_array(vec![0.5, 0.5, 1.0, 1.0, 0.2]),
                ..Default::default()
            }
            << {
                self.button(
                    "New Game",
                    self.make_callback1(|data| {
                        data.state = GameState::Playing;
                        data.board = Board::default();
                        if data.intelligence.0 == PlayerInt::AI
                            || data.intelligence.1 == PlayerInt::AI
                        {
                            ai_new_game(data.board.moves());
                            if !data.human_comes() {
                                ai_move(&mut data.board);
                            }
                        }
                    }),
                    0.8,
                );
                self.button(
                    "Load Game",
                    self.make_callback1(|data| {
                        data.state = GameState::LoadSaved;
                    }),
                    0.8,
                );
                -GridLayout {
                    col_widths: GuiDimension::relative_array(vec![1.0, 1.0]),
                    ..Default::default()
                } << {
                    -Overlay::from(Vec4::WHITE.with_w(0.5))
                        << -Text {
                            text: "Players:".to_owned(),
                            ..Default::default()
                        };
                    self.button(
                        &self.int_text(),
                        self.make_callback1(|data| {
                            data.intelligence = match data.intelligence {
                                (PlayerInt::Human, PlayerInt::Human) => {
                                    (PlayerInt::AI, PlayerInt::Human)
                                }
                                (PlayerInt::AI, PlayerInt::Human) => {
                                    (PlayerInt::Human, PlayerInt::AI)
                                }
                                (PlayerInt::Human, PlayerInt::AI) => {
                                    (PlayerInt::Human, PlayerInt::Human)
                                }
                                _ => unimplemented!(),
                            };
                        }),
                        0.8,
                    );
                };
                self.button(
                    "Exit",
                    self.make_callback3(|_data, _button, world| {
                        world.send_root(message::EXIT);
                    }),
                    0.8,
                );
                -LinearBar {
                    background: ButtonBckg::Fill(Vec4::WHITE.with_w(0.5)),
                    ..Default::default()
                };
            };
    }

    fn button(&self, text: &str, cb: GuiCallback<Button>, alpha: f32) {
        -Padding::default()
            << -Button {
                size: Vec2px::new(160.0, 50.0).into(),
                callback: cb,
                text: text.to_owned(),
                text_color: Vec4::BLACK,
                background: ButtonBckg::Fill(Vec4::WHITE.with_w(alpha)),
                ..Default::default()
            };
    }

    fn board_gui(&self, board: Board, result: GameResult) {
        let active = !result.over() && board.human_comes(self.intelligence.0, self.intelligence.1);

        -Square::default() << -Padding::absolute(24.0) << {
            -Padding::ratios(0.05, 0.0, 0.05, 0.0)
                << -Image {
                    name: "images/board_shadow".to_owned(),
                    ..Default::default()
                };

            -Padding::relative(0.05)
                << -Image::from("images/board")
                << -Padding::relative(1.0 / 32.0)
                << -GridLayout {
                    col_widths: GuiDimension::relative_array(vec![1.0; MAP_SIZE]),
                    row_heights: GuiDimension::relative_array(vec![1.0; MAP_SIZE]),
                    ..Default::default()
                }
                << {
                    let black_turn = board.next_color() == Cell::Black;
                    let over = result.over();
                    let move_count = board.moves().len();
                    let map = board.move_to_id_map();

                    for n in 0..MAP_SIZE {
                        for k in 0..MAP_SIZE {
                            let cell = board.cell(n, k);
                            let heat = board.heat[n][k];

                            if cell == Cell::Empty {
                                self.cell_gui(cell, (n, k), false, 0, black_turn, heat, active);
                            } else {
                                let id = *map.get(&(n, k)).unwrap();
                                let mut highlighted = id == move_count || id + 1 == move_count;

                                if over {
                                    highlighted = false;
                                    match result.clone() {
                                        GameResult::BlackWon(pts) | GameResult::WhiteWon(pts) => {
                                            for p in pts {
                                                if p.0 == n && p.1 == k {
                                                    highlighted = true;
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }

                                self.cell_gui(
                                    cell,
                                    (n, k),
                                    highlighted,
                                    id,
                                    black_turn,
                                    heat,
                                    active,
                                );
                            }
                        }
                    }
                };
        };
    }

    fn cell_gui(
        &self,
        cell: Cell,
        p: (usize, usize),
        highlight: bool,
        n: usize,
        black_turn: bool,
        heat: f32,
        active: bool,
    ) {
        -Padding::absolute(2.0) << {
            -Overlay {
                color: Vec4::new(1.0, 0.0, 0.0, heat),
                ..Default::default()
            };
            match cell {
                Cell::Empty if active => {
                    -Padding::relative(0.03)
                        << -Button {
                            callback: self.make_callback1(move |data| {
                                let res = data.board.put(p.0, p.1);
                                if res != GameResult::NotFinished {
                                    data.game_finished(res);
                                }
                                if GameState::Playing == data.state {
                                    if !data.human_comes() {
                                        let res = ai_move(&mut data.board);
                                        if res != GameResult::NotFinished {
                                            data.game_finished(res);
                                        }
                                    }
                                }
                            }),
                            background: ButtonBckg::Image(
                                if black_turn {
                                    "images/black"
                                } else {
                                    "images/white"
                                }
                                .to_owned(),
                                Vec4::new(1.0, 1.0, 1.0, 0.0),
                                Vec4::new(1.0, 1.0, 1.0, 0.5),
                                Vec4::new(1.0, 1.0, 1.0, 0.9),
                            ),
                            ..Default::default()
                        };
                }
                Cell::White => {
                    self.cell_img(
                        if highlight {
                            "images/white_highlighted"
                        } else {
                            "images/white"
                        },
                        Vec4::BLACK,
                        n,
                        !highlight,
                    );
                }
                Cell::Black => {
                    self.cell_img(
                        if highlight {
                            "images/black_highlighted"
                        } else {
                            "images/black"
                        },
                        Vec4::WHITE,
                        n,
                        !highlight,
                    );
                }
                _ => {}
            }
        };
    }

    fn cell_img(&self, name: &str, clr: Vec4, n: usize, show_number: bool) {
        -Padding::ratios(0.03, 0.0, 0.03, 0.0)
            << -Image {
                name: "images/stone_shadow".to_owned(),
                size: WidgetSize::relative(Vec2::new(1.1, 1.1)),
                ..Default::default()
            };
        -Padding::relative(0.03)
            << -Image::from(name)
            << if show_number {
                -Text {
                    text: format!("{}", n),
                    color: clr,
                    font_size: FontSize::relative_steps(0.5, (6.0, 32.0), 4.0),
                    ..Default::default()
                };
            }
    }

    fn game_finished(&mut self, result: GameResult) {
        self.state = GameState::Finished(result);
    }
}

#[derive(Default)]
pub struct OuterImagePrivate {
    real_size: Vec2px,
}

#[derive(Default)]
pub struct OuterImage {
    pub size: WidgetSize,
    pub name: String,
    pub mid: Vec2,
    pub private: OuterImagePrivate,
}

impl_widget_building_for!(OuterImage);
impl Widget for OuterImage {
    fn constraint(&mut self, self_constraint: WidgetConstraints) {
        self.private.real_size = self.size.to_units(self_constraint.max_size);
    }
    fn on_draw_build(&self, builder: &mut DrawBuilder) {
        let w = self.size();
        let t = builder
            .resources()
            .texture_size(&self.name)
            .unwrap_or(Vec2::new_xy(1.0));

        let r = if w.aspect() > t.aspect() {
            w.x / t.x
        } else {
            w.y / t.y
        };
        // println!("widget size: {:?}. image size: {:?}, r is {:?}", w, t, r);
        let cutout_size = w.to_pixels(1.0) / r / t;
        let midpt = (Vec2::new_xy(1.0) - cutout_size) * self.mid;

        builder.add_tex_rect(
            Rect::from_min_max(Vec2::origin(), w.to_pixels(1.0)),
            Rect::from_pos_size(midpt, cutout_size),
            &self.name,
            Vec4::WHITE,
        );
    }
    fn size(&self) -> Vec2px {
        self.private.real_size
    }
}
