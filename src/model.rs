use rand::prelude::*;
use std::time;

pub const SCREEN_WIDTH: i32 = 640;
pub const SCREEN_HEIGHT: i32 = 420;
pub const CELL_SIZE: i32 = 20;
pub const CELLS_X_LEN: i32 = SCREEN_WIDTH / CELL_SIZE;
pub const CELLS_X_MIN: i32 = 0;
pub const CELLS_X_MAX: i32 = CELLS_X_LEN - 1;
pub const CELLS_Y_LEN: i32 = SCREEN_HEIGHT / CELL_SIZE;
pub const CELLS_Y_MIN: i32 = 0;
pub const CELLS_Y_MAX: i32 = CELLS_Y_LEN - 1;

pub enum Command {
    None,
    Left,
    Right,
    Down,
    Up,
}

pub enum Direction {
    Left,
    Right,
    Down,
    Up,
}

#[derive(Debug, Default, Clone)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub struct Player {
    pub p: Point,
    pub direction: Direction,
    pub body: Vec<Point>,
}

impl Player {
    pub fn new() -> Self {
        let player = Player {
            p: Point::default(),
            direction: Direction::Up,
            body: Vec::new(),
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
        match self.direction {
            Direction::Left => self.p.x = min_max_loop_inc(CELLS_X_MIN, self.p.x - 1, CELLS_X_MAX),
            Direction::Right => self.p.x = min_max_loop_inc(CELLS_X_MIN, self.p.x + 1, CELLS_X_MAX),
            Direction::Up => self.p.y = min_max_loop_inc(CELLS_Y_MIN, self.p.y - 1, CELLS_Y_MAX),
            Direction::Down => self.p.y = min_max_loop_inc(CELLS_Y_MIN, self.p.y + 1, CELLS_Y_MAX),
        }
        println!("{} {}", self.p.x, self.p.y);
    }
}

pub struct Game {
    pub rng: StdRng,
    pub is_over: bool,
    pub frame: i32,
    pub player: Player,
    pub score: i32,
    pub requested_sounds: Vec<&'static str>,
}

impl Game {
    pub fn new() -> Self {
        let now = time::SystemTime::now();
        let timestamp = now
            .duration_since(time::UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!")
            .as_secs();
        let rng = StdRng::seed_from_u64(timestamp);

        let game = Game {
            rng: rng,
            is_over: false,
            frame: 0,
            player: Player::new(),
            score: 0,
            requested_sounds: Vec::new(),
        };

        println!("CELLS_Y_LEN = {}", CELLS_Y_LEN);
        println!("CELLS_Y_MIN = {}", CELLS_Y_MIN);
        println!("CELLS_Y_MAX = {}", CELLS_Y_MAX);

        game
    }

    pub fn update(&mut self, command: Command) {
        if self.is_over {
            return;
        }

        match command {
            Command::None => {}
            Command::Left => self.player.set_direction(Direction::Left),
            Command::Right => self.player.set_direction(Direction::Right),
            Command::Down => self.player.set_direction(Direction::Down),
            Command::Up => self.player.set_direction(Direction::Up),
        }

        if self.frame != 0 && self.frame % 15 == 0 {
            self.player.do_move();
        }

        // if self.rng.gen_bool(0.07) && self.asteroids.len() < 30 {
        //     self.spawn_asteroid();
        // }

        self.frame += 1;
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

pub fn min_max_loop_inc(min: i32, value: i32, max: i32) -> i32 {
    if value < min {
        println!("over!");
        return value + (max + 1);
    }
    if value > max {
        println!("over! 2");
        return value - (max + 1);
    }
    value
}

pub fn is_collide(x1: f32, y1: f32, w1: f32, h1: f32, x2: f32, y2: f32, w2: f32, h2: f32) -> bool {
    return (x1 <= x2 + w2 && x2 <= x1 + w1) && (y1 <= y2 + h2 && y2 <= y1 + h1);
}
