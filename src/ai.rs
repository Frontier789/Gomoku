extern crate glui;
extern crate glui_proc;
extern crate rand;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::cell::RefCell;
use std::collections::HashMap;
// use tools::*;

use board::*;
use std::cmp::*;

struct AiData {
    white_board: BoardAsNums,
    black_board: BoardAsNums,
}

impl AiData {
    fn new() -> AiData {
        AiData {
            white_board: BoardAsNums::new(Cell::White),
            black_board: BoardAsNums::new(Cell::Black),
        }
    }
}

thread_local! {
    static AIDATA_INSTANCE: RefCell<AiData> = RefCell::new(AiData::new());
}

pub fn ai_new_game(board: &Board) {
    AIDATA_INSTANCE.with(|ai_data_pers| {
        let mut ai_data = ai_data_pers.borrow_mut();

        let n = ai_data.white_board.moves.len();
        for _ in 0..n {
            ai_data.white_board.undo();
            ai_data.black_board.undo();
        }

        for p in board.moves() {
            ai_data.white_board.put(p.0, p.1);
            ai_data.black_board.put(p.0, p.1);
        }

        // println!("white's board: {:#?}", ai_data.white_board.cells);
    });
}

pub fn ai_redo(p: (usize, usize)) {
    AIDATA_INSTANCE.with(|ai_data_pers| {
        let mut ai_data = ai_data_pers.borrow_mut();

        ai_data.white_board.put(p.0, p.1);
        ai_data.black_board.put(p.0, p.1);
    });
}

pub fn ai_undo() {
    AIDATA_INSTANCE.with(|ai_data_pers| {
        let mut ai_data = ai_data_pers.borrow_mut();

        ai_data.white_board.undo();
        ai_data.black_board.undo();
    });
}

pub fn ai_move(board: &mut Board) -> GameResult {
    let pos = AIDATA_INSTANCE.with(|ai_data_pers| {
        let mut ai_data = ai_data_pers.borrow_mut();

        if let Some(m) = board.moves().last() {
            ai_data.white_board.put(m.0, m.1);
            ai_data.black_board.put(m.0, m.1);
        }

        let Move {
            value,
            pos,
            searched_count,
        } = alphabeta(&mut ai_data, true, 0.51, 3, std::f32::MIN, std::f32::MAX);

        println!(
            "AI searched through {} operations, value is {}",
            searched_count, value
        );

        // println!("black has combos: {:?}", ai_data.black_board.combos);

        ai_data.white_board.put(pos.0, pos.1);
        ai_data.black_board.put(pos.0, pos.1);

        pos
    });

    board.put(pos.0, pos.1)
}

#[derive(Copy, Clone)]
struct Move {
    value: f32,
    pos: (usize, usize),
    searched_count: usize,
}

fn alphabeta(
    ai_data: &mut AiData,
    my_turn: bool,
    mut aggression: f32,
    depth: u32,
    mut alpha: f32,
    mut beta: f32,
) -> Move {
    let black_turn = ai_data.white_board.moves.len() % 2 == 0;

    if depth == 0 {
        let white_val = ai_data.white_board.combos.eval() as f32;
        let black_val = ai_data.black_board.combos.eval() as f32;

        let (my_val, enemy_val) = if black_turn && my_turn || !black_turn && !my_turn {
            (black_val, white_val)
        } else {
            (white_val, black_val)
        };

        if !my_turn {
            aggression = 1.0 - aggression;
        }

        return Move {
            value: my_val * aggression - enemy_val * (1.0 - aggression),
            pos: (0, 0),
            searched_count: 1,
        };
    }

    if ai_data.black_board.combos.fives > 0 || ai_data.white_board.combos.fives > 0 {
        // println!("I run on an over");
        return Move {
            value: if my_turn { -10000.0 } else { 10000.0 },
            pos: (0, 0),
            searched_count: 0,
        };
    }

    // if ai_data.black_board.combos.fives > 0 {
    //     return Move {
    //         value: -10000.0,
    //         pos: (0,0),
    //         searched_count: 0
    //     };
    // }

    // if ai_data.white_board.combos.fives > 0 {
    //     return Move {
    //         value: 10000.0,
    //         pos: (0,0),
    //         searched_count: 0
    //     };
    // }

    let mut close = [[false; MAP_SIZE]; MAP_SIZE];
    {
        let mut close1 = [[false; MAP_SIZE]; MAP_SIZE];

        for n in 0..MAP_SIZE {
            for k in 0..MAP_SIZE {
                if ai_data.white_board.cells[n][k] != Cell::Empty {
                    close1[n][k] = true;
                    if n > 1 {
                        close1[n - 1][k] = true;
                    }
                    if n > 2 {
                        close1[n - 2][k] = true;
                    }
                    if n + 1 < MAP_SIZE {
                        close1[n + 1][k] = true;
                    }
                    if n + 2 < MAP_SIZE {
                        close1[n + 2][k] = true;
                    }
                }
            }
        }

        for n in 0..MAP_SIZE {
            for k in 0..MAP_SIZE {
                if close1[n][k] {
                    close[n][k] = true;
                }
                if k > 0 && close1[n][k - 1] {
                    close[n][k] = true;
                }
                if k > 1 && close1[n][k - 2] {
                    close[n][k] = true;
                }
                if k + 1 < MAP_SIZE && close1[n][k + 1] {
                    close[n][k] = true;
                }
                if k + 2 < MAP_SIZE && close1[n][k + 2] {
                    close[n][k] = true;
                }
            }
        }
    }

    close[MAP_SIZE / 2][MAP_SIZE / 2] = true;

    let mut mx_value = std::f32::MIN;
    let mut mx_pos = (0, 0);
    let mut mn_value = std::f32::MAX;
    let mut mn_pos = (0, 0);
    let mut searched = 0;

    let mut possible_moves = vec![];

    for n in 0..MAP_SIZE {
        for k in 0..MAP_SIZE {
            if ai_data.white_board.cells[n][k] == Cell::Empty && close[n][k] {
                possible_moves.push((n, k));
            }
        }
    }

    possible_moves.shuffle(&mut thread_rng());

    for (n, k) in possible_moves {
        ai_data.white_board.put(n, k);
        ai_data.black_board.put(n, k);

        let Move {
            value,
            pos: _,
            searched_count,
        } = alphabeta(ai_data, !my_turn, aggression, depth - 1, alpha, beta);
        searched += searched_count;

        if my_turn && alpha < value {
            alpha = value;
        }
        if !my_turn && beta > value {
            beta = value;
        }

        if mx_value < value {
            mx_value = value;
            mx_pos = (n, k);
        }
        if mn_value > value {
            mn_value = value;
            mn_pos = (n, k);
        }

        ai_data.black_board.undo();
        ai_data.white_board.undo();

        if alpha >= beta {
            break;
        }
    }

    if my_turn {
        Move {
            value: mx_value,
            pos: mx_pos,
            searched_count: searched,
        }
    } else {
        Move {
            value: mn_value,
            pos: mn_pos,
            searched_count: searched,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Eq)]
struct Combinations {
    fives: u8,
    open_fours: u8,
    half_open_fours: u8,
    open_threes: u8,
    half_open_threes: u8,
    open_twos: u8,
}

impl PartialEq for Combinations {
    fn eq(&self, other: &Self) -> bool {
        if min(self.fives, other.fives) > 0 {
            return self.fives == other.fives;
        }
        if min(self.open_fours, other.open_fours) > 0 {
            return self.open_fours == other.open_fours;
        }
        self.eval() == other.eval()
    }
}
impl PartialOrd for Combinations {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if min(self.fives, other.fives) > 0 {
            return self.fives.partial_cmp(&other.fives);
        }
        if min(self.open_fours, other.open_fours) > 0 {
            return self.open_fours.partial_cmp(&other.open_fours);
        }
        self.eval().partial_cmp(&other.eval())
    }
}

impl std::ops::AddAssign for Combinations {
    fn add_assign(&mut self, rhs: Combinations) {
        self.fives += rhs.fives;
        self.open_fours += rhs.open_fours;
        self.half_open_fours += rhs.half_open_fours;
        self.open_threes += rhs.open_threes;
        self.half_open_threes += rhs.half_open_threes;
        self.open_twos += rhs.open_twos;
    }
}

impl std::ops::SubAssign for Combinations {
    fn sub_assign(&mut self, rhs: Combinations) {
        self.fives -= rhs.fives;
        self.open_fours -= rhs.open_fours;
        self.half_open_fours -= rhs.half_open_fours;
        self.open_threes -= rhs.open_threes;
        self.half_open_threes -= rhs.half_open_threes;
        self.open_twos -= rhs.open_twos;
    }
}

impl Combinations {
    fn eval(&self) -> f64 {
        self.fives as f64 * 100.0
            + self.open_fours as f64 * 10.0
            + self.half_open_fours as f64 * 5.0
            + self.open_threes as f64 * 1.0
            + self.half_open_threes as f64 * 0.1
            + self.open_twos as f64 * 0.01
    }
}

#[derive(Debug, Default)]
struct BoardAsNums {
    rows: [u32; MAP_SIZE],
    cols: [u32; MAP_SIZE],
    rds: [u32; MAP_SIZE * 2 - 1],
    rus: [u32; MAP_SIZE * 2 - 1],
    cells: [[Cell; MAP_SIZE]; MAP_SIZE],
    moves: Vec<(usize, usize)>,
    combos: Combinations,
    player: Cell,
    num_to_combos: [HashMap<u32, Combinations>; MAP_SIZE + 1],
}

fn proj_rds(x: usize, y: usize) -> (i32, i32) {
    if x < y {
        (0, (y - x) as i32)
    } else {
        ((x - y) as i32, 0)
    }
}
fn len_rds(x: usize, y: usize) -> usize {
    MAP_SIZE - (x as i32 - y as i32).abs() as usize
}
fn index_rds(x: usize, y: usize) -> usize {
    (x as i32 - y as i32 + MAP_SIZE as i32 - 1) as usize
}
fn adv_rds(x: usize, y: usize) -> usize {
    min(x, y)
}

fn proj_rus(x: usize, y: usize) -> (i32, i32) {
    if x + y < MAP_SIZE - 1 {
        (0, (y + x) as i32)
    } else {
        ((x + y - (MAP_SIZE - 1)) as i32, (MAP_SIZE - 1) as i32)
    }
}
fn len_rus(x: usize, y: usize) -> usize {
    MAP_SIZE - ((x + y) as i32 - (MAP_SIZE - 1) as i32).abs() as usize
}
fn index_rus(x: usize, y: usize) -> usize {
    x + y
}
fn adv_rus(x: usize, y: usize) -> usize {
    min(x, MAP_SIZE - 1 - y)
}

impl BoardAsNums {
    pub fn new(player: Cell) -> BoardAsNums {
        BoardAsNums {
            player,
            ..Default::default()
        }
    }
    pub fn put(&mut self, x: usize, y: usize) {
        let colmod = (self.moves.len() % 2) as u32 + 1;
        self.cells[x][y] = if self.moves.len() % 2 == 0 {
            Cell::Black
        } else {
            Cell::White
        };
        self.moves.push((x, y));

        let sub = self.evaluate((0, y as i32), (1, 0), self.cols[y], MAP_SIZE);
        self.combos -= sub;
        self.cols[y] += 3u32.pow(x as u32) * colmod;
        let add = self.evaluate((0, y as i32), (1, 0), self.cols[y], MAP_SIZE);
        self.combos += add;

        let sub = self.evaluate((x as i32, 0), (0, 1), self.rows[x], MAP_SIZE);
        self.combos -= sub;
        self.rows[x] += 3u32.pow(y as u32) * colmod;
        let add = self.evaluate((x as i32, 0), (0, 1), self.rows[x], MAP_SIZE);
        self.combos += add;

        let sub = self.evaluate(
            proj_rds(x, y),
            (1, 1),
            self.rds[index_rds(x, y)],
            len_rds(x, y),
        );
        self.combos -= sub;
        self.rds[index_rds(x, y)] += 3u32.pow(adv_rds(x, y) as u32) * colmod;
        let add = self.evaluate(
            proj_rds(x, y),
            (1, 1),
            self.rds[index_rds(x, y)],
            len_rds(x, y),
        );
        self.combos += add;

        let sub = self.evaluate(
            proj_rus(x, y),
            (1, -1),
            self.rus[index_rus(x, y)],
            len_rus(x, y),
        );
        self.combos -= sub;
        self.rus[index_rus(x, y)] += 3u32.pow(adv_rus(x, y) as u32) * colmod;
        let add = self.evaluate(
            proj_rus(x, y),
            (1, -1),
            self.rus[index_rus(x, y)],
            len_rus(x, y),
        );
        self.combos += add;
    }
    pub fn undo(&mut self) {
        if let Some((x, y)) = self.moves.pop() {
            let colmod = (self.moves.len() % 2) as u32 + 1;
            self.cells[x][y] = Cell::Empty;

            let sub = self.evaluate((0, y as i32), (1, 0), self.cols[y], MAP_SIZE);
            self.combos -= sub;
            self.cols[y] -= 3u32.pow(x as u32) * colmod;
            let add = self.evaluate((0, y as i32), (1, 0), self.cols[y], MAP_SIZE);
            self.combos += add;

            let sub = self.evaluate((x as i32, 0), (0, 1), self.rows[x], MAP_SIZE);
            self.combos -= sub;
            self.rows[x] -= 3u32.pow(y as u32) * colmod;
            let add = self.evaluate((x as i32, 0), (0, 1), self.rows[x], MAP_SIZE);
            self.combos += add;

            let sub = self.evaluate(
                proj_rds(x, y),
                (1, 1),
                self.rds[index_rds(x, y)],
                len_rds(x, y),
            );
            self.combos -= sub;
            self.rds[index_rds(x, y)] -= 3u32.pow(adv_rds(x, y) as u32) * colmod;
            let add = self.evaluate(
                proj_rds(x, y),
                (1, 1),
                self.rds[index_rds(x, y)],
                len_rds(x, y),
            );
            self.combos += add;

            let sub = self.evaluate(
                proj_rus(x, y),
                (1, -1),
                self.rus[index_rus(x, y)],
                len_rus(x, y),
            );
            self.combos -= sub;
            self.rus[index_rus(x, y)] -= 3u32.pow(adv_rus(x, y) as u32) * colmod;
            let add = self.evaluate(
                proj_rus(x, y),
                (1, -1),
                self.rus[index_rus(x, y)],
                len_rus(x, y),
            );
            self.combos += add;
        }
    }
    pub fn evaluate(&mut self, p: (i32, i32), v: (i32, i32), num: u32, len: usize) -> Combinations {
        if let Some(v) = self.num_to_combos[len].get(&num) {
            *v
        } else {
            let line = self.line(p, v).collect::<Vec<Cell>>();
            let opponent = self.player.opponent();
            let mut ret = Combinations::default();
            let player = self.player;
            for i in 0..len {
                if i + 5 <= len
                    && line[i] == player
                    && line[i + 1] == player
                    && line[i + 2] == player
                    && line[i + 3] == player
                    && line[i + 4] == player
                {
                    ret.fives += 1;
                }
                if i + 6 <= len
                    && line[i] == Cell::Empty
                    && line[i + 1] == player
                    && line[i + 2] == player
                    && line[i + 3] == player
                    && line[i + 4] == player
                    && line[i + 5] == Cell::Empty
                {
                    ret.open_fours += 1;
                }
                if i + 6 <= len
                    && line[i] == opponent
                    && line[i + 1] == player
                    && line[i + 2] == player
                    && line[i + 3] == player
                    && line[i + 4] == player
                    && line[i + 5] == Cell::Empty
                {
                    ret.half_open_fours += 1;
                }
                if i + 6 <= len
                    && line[i] == opponent
                    && line[i + 1] == player
                    && line[i + 2] == player
                    && line[i + 3] == player
                    && line[i + 4] == Cell::Empty
                    && line[i + 5] == player
                {
                    ret.half_open_fours += 1;
                }
                if i + 6 <= len
                    && line[i] == opponent
                    && line[i + 1] == player
                    && line[i + 2] == player
                    && line[i + 3] == Cell::Empty
                    && line[i + 4] == player
                    && line[i + 5] == player
                {
                    ret.half_open_fours += 1;
                }
                if i + 6 <= len
                    && line[i] == opponent
                    && line[i + 1] == player
                    && line[i + 2] == Cell::Empty
                    && line[i + 3] == player
                    && line[i + 4] == player
                    && line[i + 5] == player
                {
                    ret.half_open_fours += 1;
                }
                if i + 6 <= len
                    && line[i] == Cell::Empty
                    && line[i + 1] == player
                    && line[i + 2] == player
                    && line[i + 3] == player
                    && line[i + 4] == player
                    && line[i + 5] == opponent
                {
                    ret.half_open_fours += 1;
                }
                if i + 6 <= len
                    && line[i] == player
                    && line[i + 1] == Cell::Empty
                    && line[i + 2] == player
                    && line[i + 3] == player
                    && line[i + 4] == player
                    && line[i + 5] == opponent
                {
                    ret.half_open_fours += 1;
                }
                if i + 6 <= len
                    && line[i] == player
                    && line[i + 1] == player
                    && line[i + 2] == Cell::Empty
                    && line[i + 3] == player
                    && line[i + 4] == player
                    && line[i + 5] == opponent
                {
                    ret.half_open_fours += 1;
                }
                if i + 6 <= len
                    && line[i] == player
                    && line[i + 1] == player
                    && line[i + 2] == player
                    && line[i + 3] == Cell::Empty
                    && line[i + 4] == player
                    && line[i + 5] == opponent
                {
                    ret.half_open_fours += 1;
                }
                if len >= 5
                    && i == 0
                    && line[i] == player
                    && line[i + 1] == player
                    && line[i + 2] == player
                    && line[i + 3] == player
                    && line[i + 4] == Cell::Empty
                {
                    ret.half_open_fours += 1;
                }
                if len >= 5
                    && i == len - 5
                    && line[i] == Cell::Empty
                    && line[i + 1] == player
                    && line[i + 2] == player
                    && line[i + 3] == player
                    && line[i + 4] == player
                {
                    ret.half_open_fours += 1;
                }
                if i + 5 <= len
                    && line[i] == Cell::Empty
                    && line[i + 1] == player
                    && line[i + 2] == player
                    && line[i + 3] == player
                    && line[i + 4] == Cell::Empty
                {
                    ret.open_threes += 1;
                }
                if i + 6 <= len
                    && line[i] == Cell::Empty
                    && line[i + 1] == player
                    && line[i + 2] == player
                    && line[i + 3] == Cell::Empty
                    && line[i + 4] == player
                    && line[i + 5] == Cell::Empty
                {
                    ret.open_threes += 1;
                }
                if i + 6 <= len
                    && line[i] == Cell::Empty
                    && line[i + 1] == player
                    && line[i + 2] == Cell::Empty
                    && line[i + 3] == player
                    && line[i + 4] == player
                    && line[i + 5] == Cell::Empty
                {
                    ret.open_threes += 1;
                }
                if i + 5 <= len
                    && line[i] == Cell::Empty
                    && line[i + 1] == player
                    && line[i + 2] == player
                    && line[i + 3] == player
                    && line[i + 4] == opponent
                {
                    ret.half_open_threes += 1;
                }
                if i + 6 <= len
                    && line[i] == Cell::Empty
                    && line[i + 1] == player
                    && line[i + 2] == Cell::Empty
                    && line[i + 3] == player
                    && line[i + 4] == player
                    && line[i + 5] == opponent
                {
                    ret.half_open_threes += 1;
                }
                if i + 6 <= len
                    && line[i] == Cell::Empty
                    && line[i + 1] == player
                    && line[i + 2] == player
                    && line[i + 3] == Cell::Empty
                    && line[i + 4] == player
                    && line[i + 5] == opponent
                {
                    ret.half_open_threes += 1;
                }
                if i + 5 <= len
                    && line[i] == opponent
                    && line[i + 1] == player
                    && line[i + 2] == player
                    && line[i + 3] == player
                    && line[i + 4] == Cell::Empty
                {
                    ret.half_open_threes += 1;
                }
                if i + 6 <= len
                    && line[i] == opponent
                    && line[i + 1] == player
                    && line[i + 2] == player
                    && line[i + 3] == Cell::Empty
                    && line[i + 4] == player
                    && line[i + 5] == Cell::Empty
                {
                    ret.half_open_threes += 1;
                }
                if i + 6 <= len
                    && line[i] == opponent
                    && line[i + 1] == player
                    && line[i + 2] == Cell::Empty
                    && line[i + 3] == player
                    && line[i + 4] == player
                    && line[i + 5] == Cell::Empty
                {
                    ret.half_open_threes += 1;
                }
                if len >= 4
                    && i == 0
                    && line[i] == player
                    && line[i + 1] == player
                    && line[i + 2] == player
                    && line[i + 3] == Cell::Empty
                {
                    ret.half_open_threes += 1;
                }
                if len >= 4
                    && i == len - 4
                    && line[i] == Cell::Empty
                    && line[i + 1] == player
                    && line[i + 2] == player
                    && line[i + 3] == player
                {
                    ret.half_open_threes += 1;
                }
                if i + 4 <= len
                    && line[i] == Cell::Empty
                    && line[i + 1] == player
                    && line[i + 2] == player
                    && line[i + 3] == Cell::Empty
                {
                    ret.open_twos += 1;
                }
            }
            self.num_to_combos[len].insert(num, ret);
            ret
        }
    }
    pub fn line<'a>(&'a self, pos: (i32, i32), dir: (i32, i32)) -> BoardLineIterator<'a> {
        BoardLineIterator {
            p: pos,
            v: dir,
            cells: &self.cells,
        }
    }
}
