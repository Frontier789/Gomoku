extern crate chrono;
extern crate glui;
extern crate glui_proc;
extern crate rand;
extern crate serde_json;
use self::chrono::{Datelike, Timelike};
use gamestate::PlayerInt;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;

pub const MAP_SIZE: usize = 15;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Cell {
    Empty,
    Black,
    White,
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Empty
    }
}

impl Cell {
    pub fn opponent(self) -> Cell {
        match self {
            Cell::Black => Cell::White,
            Cell::White => Cell::Black,
            _ => Cell::Empty,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Board {
    cells: [[Cell; MAP_SIZE]; MAP_SIZE],
    pub heat: [[f32; MAP_SIZE]; MAP_SIZE],
    moves: Vec<(usize, usize)>,
    redo_stack: Vec<(usize, usize)>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum GameResult {
    NotFinished,
    BlackWon(Vec<(usize, usize)>),
    WhiteWon(Vec<(usize, usize)>),
    Draw,
}

impl GameResult {
    pub fn or(self, res: GameResult) -> GameResult {
        use self::GameResult::*;
        match (self, res) {
            (NotFinished, Draw) => NotFinished,
            (NotFinished, r) => r,
            (Draw, r) => r,
            (r, _) => r,
        }
    }
    pub fn win_text(&self) -> String {
        match self {
            GameResult::BlackWon(_) => "Black won",
            GameResult::WhiteWon(_) => "White won",
            GameResult::Draw => "It's a draw",
            _ => "",
        }
        .to_owned()
    }
    pub fn over(&self) -> bool {
        match self {
            GameResult::BlackWon(_) | GameResult::WhiteWon(_) | GameResult::Draw => true,
            _ => false,
        }
    }
}

pub struct BoardLineIterator<'a> {
    pub p: (i32, i32),
    pub v: (i32, i32),
    pub cells: &'a [[Cell; MAP_SIZE]; MAP_SIZE],
}

impl<'a> Iterator for BoardLineIterator<'a> {
    type Item = Cell;
    fn next(&mut self) -> Option<Self::Item> {
        if self.p.0 < 0
            || self.p.1 < 0
            || self.p.0 >= MAP_SIZE as i32
            || self.p.1 >= MAP_SIZE as i32
        {
            return None;
        }
        let cell = self.cells[self.p.0 as usize][self.p.1 as usize];
        self.p.0 += self.v.0;
        self.p.1 += self.v.1;
        Some(cell)
    }
}

impl<'a> BoardLineIterator<'a> {
    pub fn last5(&self) -> Vec<(usize, usize)> {
        let (p, v) = (self.p, self.v);
        vec![
            (p.0 - v.0 * 1, p.1 - v.1 * 1),
            (p.0 - v.0 * 2, p.1 - v.1 * 2),
            (p.0 - v.0 * 3, p.1 - v.1 * 3),
            (p.0 - v.0 * 4, p.1 - v.1 * 4),
            (p.0 - v.0 * 5, p.1 - v.1 * 5),
        ]
        .iter()
        .map(|(x, y)| (*x as usize, *y as usize))
        .collect()
    }
}

impl Board {
    pub fn can_undo(&self) -> bool {
        !self.moves.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
    pub fn moves(&self) -> &Vec<(usize, usize)> {
        &self.moves
    }
    pub fn next_color(&self) -> Cell {
        let n = self.moves.len();
        if n % 2 == 0 {
            Cell::Black
        } else {
            Cell::White
        }
    }
    pub fn move_to_id_map(&self) -> HashMap<(usize, usize), usize> {
        let mut map = HashMap::new();
        for i in 0..self.moves.len() {
            map.insert(self.moves[i], i + 1);
        }
        map
    }
    pub fn human_comes(&self, black_player: PlayerInt, white_player: PlayerInt) -> bool {
        [black_player, white_player][self.moves.len() % 2] == PlayerInt::Human
    }
    pub fn cell(&self, x: usize, y: usize) -> Cell {
        self.cells[x][y]
    }
    pub fn put_mayredo(&mut self, x: usize, y: usize, clear_redo: bool) -> GameResult {
        self.cells[x][y] = self.next_color();
        self.moves.push((x, y));

        if clear_redo {
            self.redo_stack = vec![];
        }

        self.result()
    }
    pub fn put(&mut self, x: usize, y: usize) -> GameResult {
        self.put_mayredo(x, y, true)
    }
    pub fn undo(&mut self) {
        if let Some(p) = self.moves.pop() {
            self.cells[p.0][p.1] = Cell::Empty;
            self.redo_stack.push(p);
        }
    }
    pub fn redo_step(&mut self) -> Option<(usize, usize)> {
        self.redo_stack.last().map(|p| *p)
    }
    pub fn redo(&mut self) -> GameResult {
        if let Some(p) = self.redo_stack.pop() {
            self.put_mayredo(p.0, p.1, false)
        } else {
            panic!("Redoing without anything to redo!")
        }
    }
    fn check_line(mut line: BoardLineIterator) -> GameResult {
        use Cell::*;
        let mut stack = Vec::<Cell>::new();
        let mut has_space = false;
        while let Some(cell) = line.next() {
            match (cell, stack.last()) {
                (Empty, _) => {
                    stack.clear();
                    has_space = true;
                }
                (Black, Some(Black)) | (Black, None) => {
                    stack.push(Black);
                    if stack.len() == 5 {
                        return GameResult::BlackWon(line.last5());
                    }
                }
                (Black, _) => {
                    stack.clear();
                    stack.push(Black);
                }
                (White, Some(White)) | (White, None) => {
                    stack.push(White);
                    if stack.len() == 5 {
                        return GameResult::WhiteWon(line.last5());
                    }
                }
                (White, _) => {
                    stack.clear();
                    stack.push(White);
                }
            }
        }
        if has_space {
            GameResult::NotFinished
        } else {
            GameResult::Draw
        }
    }
    pub fn line(&self, pos: (i32, i32), dir: (i32, i32)) -> BoardLineIterator {
        BoardLineIterator {
            p: pos,
            v: dir,
            cells: &self.cells,
        }
    }
    pub fn result(&self) -> GameResult {
        let mut res = GameResult::Draw;
        for n in 0..MAP_SIZE {
            res = res.or(Self::check_line(self.line((0, n as i32), (1, 0))));
            res = res.or(Self::check_line(self.line((n as i32, 0), (0, 1))));

            res = res.or(Self::check_line(self.line((0, n as i32), (1, 1))));
            res = res.or(Self::check_line(self.line((0, n as i32), (1, -1))));
        }
        for n in 1..MAP_SIZE {
            res = res.or(Self::check_line(self.line((n as i32, 0), (1, 1))));
            res = res.or(Self::check_line(
                self.line((n as i32, (MAP_SIZE - 1) as i32), (1, -1)),
            ));
        }
        res
    }
    pub fn load_from_string(&mut self, moves: &str) -> serde_json::Result<()> {
        let moves: Vec<(usize, usize)> = serde_json::from_str(moves)?;

        *self = Board::default();
        for m in moves.iter() {
            self.put(m.0, m.1);
        }

        Ok(())
    }
    pub fn append_game_to_saves(&self) {
        let str = serde_json::to_string(&self.moves).unwrap_or_default();
        let now = chrono::Local::now();
        let (is_pm, hour) = now.hour12();
        let (_, year) = now.year_ce();

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("data/games.txt")
            .unwrap();

        write!(
            file,
            "{}-{:02}-{:02} {:02}:{:02}:{:02} \n{}\n",
            year,
            now.month(),
            now.day(),
            hour + if is_pm { 12 } else { 0 },
            now.minute(),
            now.second(),
            str,
        )
        .unwrap();
    }
}
