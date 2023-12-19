use rand::prelude::*;
use std::{collections::HashMap, time};

pub const SCREEN_WIDTH: i32 = 420;
pub const SCREEN_HEIGHT: i32 = 420;
pub const CELL_SIZE: i32 = 20;
pub const INFO_HEIGHT: i32 = 20;
pub const CELLS_X_LEN: i32 = SCREEN_WIDTH / CELL_SIZE;
pub const CELLS_X_MIN: i32 = 0;
pub const CELLS_X_MAX: i32 = CELLS_X_LEN - 1;
pub const CELLS_Y_LEN: i32 = (SCREEN_HEIGHT - INFO_HEIGHT) / CELL_SIZE;
pub const CELLS_Y_MIN: i32 = 0;
pub const CELLS_Y_MAX: i32 = CELLS_Y_LEN - 1;
pub const ENERGY_MAX: i32 = 100;

pub enum Command {
    None,
    Left,
    Right,
    Down,
    Up,
}

#[derive(Clone)]
pub enum Direction {
    Left,
    Right,
    Down,
    Up,
}

impl Direction {
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Up => Direction::Down,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FoodColor {
    White,
    Red,
    Yellow,
    Blue,
}

impl FoodColor {
    pub fn all() -> Vec<Self> {
        vec![
            FoodColor::White,
            FoodColor::Red,
            FoodColor::Yellow,
            FoodColor::Blue,
        ]
    }

    pub fn energy(&self) -> i32 {
        match self {
            FoodColor::White => 0,
            FoodColor::Red => 20,
            FoodColor::Yellow => 10,
            FoodColor::Blue => 5,
        }
    }
}

pub struct Food {
    pub color: FoodColor,
    pub p: Point,
    pub is_exist: bool,
}

pub struct Poo {
    pub p: Point,
    pub is_exist: bool,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Point {
            x: (CELLS_X_LEN + x) % CELLS_X_LEN,
            y: (CELLS_Y_LEN + y) % CELLS_Y_LEN,
        }
    }

    pub fn neighbor(&self, direction: Direction) -> Self {
        match direction {
            Direction::Left => Self::new(self.x - 1, self.y),
            Direction::Right => Self::new(self.x + 1, self.y),
            Direction::Up => Self::new(self.x, self.y - 1),
            Direction::Down => Self::new(self.x, self.y + 1),
        }
    }
}

pub struct Player {
    pub p: Point,
    pub direction: Direction,
    pub bodies: Vec<Point>,
    pub energy: i32,
}

impl Player {
    pub fn new() -> Self {
        let player = Player {
            p: Point::new(CELLS_X_LEN / 2, CELLS_Y_LEN / 2),
            direction: Direction::Up,
            bodies: Vec::new(),
            energy: ENERGY_MAX,
        };
        player
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }

    pub fn get_angle(&self) -> f32 {
        match self.direction {
            Direction::Left => 270.0,
            Direction::Right => 90.0,
            Direction::Down => 180.0,
            Direction::Up => 0.0,
        }
    }

    pub fn do_move(&mut self) {
        self.energy -= 1;

        if self.bodies.len() >= 1 {
            let mut i = self.bodies.len() - 1;
            while i >= 1 {
                self.bodies[i] = self.bodies[i - 1].clone();
                i -= 1;
            }

            self.bodies[0] = self.p.clone();
        }

        self.p = self.p.neighbor(self.direction.clone());
    }

    pub fn grow(&mut self) {
        let new_pos = match self.bodies.len() {
            0 => self.p.neighbor(self.direction.opposite()),
            1 => {
                let direction = get_direction(self.p.clone(), self.bodies[0].clone());
                self.bodies[0].neighbor(direction)
            }
            _ => {
                let direction = get_direction(
                    self.bodies[self.bodies.len() - 2].clone(),
                    self.bodies[self.bodies.len() - 1].clone(),
                );
                let last_pos = self.bodies.last().unwrap();
                last_pos.neighbor(direction)
            }
        };
        self.bodies.push(new_pos);
    }

    pub fn shrink(&mut self) {
        self.bodies.pop();
    }
}

// p1からp2への向きを返す
fn get_direction(p1: Point, p2: Point) -> Direction {
    if p1.x < p2.x {
        Direction::Right
    } else if p1.x > p2.x {
        Direction::Left
    } else if p1.y < p2.y {
        Direction::Down
    } else {
        Direction::Up
    }
}

pub struct Game {
    pub rng: StdRng,
    pub is_over: bool,
    pub frame: i32,
    pub player: Player,
    pub score: i32,
    pub requested_sounds: Vec<&'static str>,
    pub ate_counts: HashMap<FoodColor, i32>,
    pub foods: Vec<Food>,
    pub poos: Vec<Poo>,
    pub poo_spawn_frame: i32,
    pub ate_count: i32,
}

impl Game {
    pub fn new() -> Self {
        let now = time::SystemTime::now();
        let timestamp = now
            .duration_since(time::UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!")
            .as_secs();
        let rng = StdRng::seed_from_u64(timestamp);

        let mut game = Game {
            rng: rng,
            is_over: false,
            frame: 0,
            player: Player::new(),
            score: 0,
            requested_sounds: Vec::new(),
            ate_counts: HashMap::new(),
            foods: Vec::new(),
            poos: Vec::new(),
            poo_spawn_frame: -1,
            ate_count: 0,
        };

        for color in FoodColor::all() {
            game.ate_counts.insert(color, 0);
        }

        for y in CELLS_Y_MIN..=CELLS_Y_MAX {
            for x in CELLS_X_MIN..=CELLS_X_MAX {
                game.foods.push(Food {
                    color: FoodColor::Red,
                    p: Point { x: x, y: y },
                    is_exist: false,
                });
            }
        }

        for y in CELLS_Y_MIN..=CELLS_Y_MAX {
            for x in CELLS_X_MIN..=CELLS_X_MAX {
                game.poos.push(Poo {
                    p: Point { x: x, y: y },
                    is_exist: false,
                });
            }
        }

        game
    }

    pub fn update(&mut self, command: Command) {
        if self.is_over {
            return;
        }

        match command {
            Command::None => {}
            Command::Left => {
                self.player.set_direction(Direction::Left);
                self.requested_sounds.push("e4.wav");
            }
            Command::Right => {
                self.player.set_direction(Direction::Right);
                self.requested_sounds.push("d4.wav");
            }
            Command::Down => {
                self.player.set_direction(Direction::Down);
                self.requested_sounds.push("a4.wav");
            }
            Command::Up => {
                self.player.set_direction(Direction::Up);
                self.requested_sounds.push("g4.wav");
            }
        }

        if self.frame != 0 && self.frame % 8 == 0 {
            self.player.do_move();
        }

        if self.frame != 0 && self.frame % 30 == 0 && self.foods_count() < 5 {
            self.spawn_food();
        }

        if self.frame == self.poo_spawn_frame {
            self.spawn_poo();
        }

        for food in &mut self.foods {
            if food.is_exist {
                if food.p == self.player.p {
                    food.is_exist = false;
                    self.ate_counts
                        .insert(food.color.clone(), self.ate_counts[&food.color] + 1);
                    if food.color == FoodColor::White {
                        self.player.shrink();
                        self.requested_sounds.push("shrink.wav");
                    } else {
                        self.player.energy =
                            clamp(0, self.player.energy + food.color.energy(), ENERGY_MAX);
                        self.player.grow();
                        self.ate_count += 1;
                        if self.ate_count % 3 == 0 {
                            self.poo_spawn_frame = self.frame + 60; // 指定フレームにうんこを生み出す
                        }
                        self.requested_sounds.push("eat.wav");
                    }
                }
            }
        }

        for poo in &mut self.poos {
            if poo.is_exist {
                if poo.p == self.player.p {
                    poo.is_exist = false;
                    self.is_over = true;
                    self.requested_sounds.push("crash.wav");
                }

                // うんこと重なっている食べ物は消す
                for food in &mut self.foods {
                    if food.is_exist && food.p == poo.p {
                        food.is_exist = false;
                    }
                }
            }
        }

        for body in &self.player.bodies {
            if *body == self.player.p {
                self.is_over = true;
                self.requested_sounds.push("crash.wav");
            }
        }

        if self.player.energy < 0 {
            self.is_over = true;
            self.requested_sounds.push("crash.wav");
        }

        self.frame += 1;
        self.score = self.frame / 30;
    }

    fn foods_count(&self) -> usize {
        self.foods.iter().filter(|x| x.is_exist).count()
    }

    fn spawn_food(&mut self) {
        let i: usize = self.rng.gen_range(0..self.foods.len());
        if self.foods[i].is_exist {
            return;
        }
        self.foods[i].is_exist = true;

        let r: i32 = self.rng.gen_range(0..100);
        self.foods[i].color = if r < 40 {
            FoodColor::Blue
        } else if r < 75 {
            FoodColor::Yellow
        } else if r < 95 {
            FoodColor::Red
        } else {
            FoodColor::White
        };
    }

    fn spawn_poo(&mut self) {
        let pos;
        if self.player.bodies.len() > 0 {
            pos = self.player.bodies.last().unwrap().clone();
        } else {
            pos = self.player.p.neighbor(self.player.direction.opposite());
        }

        for poo in &mut self.poos {
            if !poo.is_exist {
                poo.p = pos.clone();
                poo.is_exist = true;
                break;
            }
        }
    }
}

fn clamp<T: PartialOrd>(min: T, value: T, max: T) -> T {
    if value < min {
        return min;
    }
    if value > max {
        return max;
    }
    value
}
