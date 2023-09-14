use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use std::thread;
use std::time::{Duration, Instant};

// length snake will start at
const MIN_SNAKE_LEN: usize = 3;
// might make these changeable one day
const BOARD_SIZE: Vec2 = Vec2 { x: 21, y: 17 };
const SNAKE_SPEED: f32 = 0.2;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("snake game", 600, 800)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        let settings = Settings {
            board_size: BOARD_SIZE,
            snake_speed: SNAKE_SPEED,
        };
        // Initialize menu

        // 'menu: loop {}

        // Start game
        let mut game = Game::new(settings);

        // Game loop:
        // * check for player input and change snake direction accordingly
        // * move snake, check for apples or game over
        // * draw canvas
        // * sleep until next snake move
        'game: loop {
            let instant = Instant::now();

            let mut snake_moved = false;
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'running,
                    Event::KeyDown {
                        keycode: Some(keycode),
                        ..
                    } => {
                        if snake_moved {
                            continue;
                        }
                        let d = match keycode {
                            Keycode::W | Keycode::Up => Some(Direction::Up),
                            Keycode::S | Keycode::Down => Some(Direction::Down),
                            Keycode::A | Keycode::Left => Some(Direction::Left),
                            Keycode::D | Keycode::Right => Some(Direction::Right),
                            _ => None,
                        };
                        if let Some(d) = d {
                            if game.snake[0].direction.opposite() != d {
                                snake_moved = true;
                                game.snake[0].direction = d;
                            }
                        }
                    }
                    _ => (),
                }
            }
            if !game.move_snake() {
                break 'game;
            }
            game.draw_canvas(&mut canvas)?;
            thread::sleep(Duration::from_secs_f32(
                game.settings.snake_speed - instant.elapsed().as_secs_f32(),
            ));
        }
        // Game over
        println!("Game over!");

        // temporary
        break 'running;
    }
    Ok(())
}

// TODO
// * menu
// * game over screen
// * change settings in menu or settings page
// * obstacles

struct Game {
    score: u16,
    snake: Vec<SnakeBody>,
    apple: Vec2,
    rng: SmallRng,
    settings: Settings,
}

struct Settings {
    board_size: Vec2,
    // time between snake movements, smaller is faster
    snake_speed: f32,
}

impl Game {
    fn new(settings: Settings) -> Self {
        let snake = vec![SnakeBody {
            pos: Vec2 {
                x: settings.board_size.x / 2,
                y: settings.board_size.y / 2,
            },
            direction: Direction::Up,
        }];
        let mut rng = SmallRng::from_entropy();
        let apple = Vec2 {
            x: rng.gen_range(0..settings.board_size.x),
            y: rng.gen_range(0..settings.board_size.y),
        };
        Game {
            score: 0,
            snake,
            apple,
            rng,
            settings,
        }
    }
    fn place_apple(&mut self) {
        'apple: loop {
            let apple_pos = Vec2 {
                x: self.rng.gen_range(1..=self.settings.board_size.x),
                y: self.rng.gen_range(1..=self.settings.board_size.y),
            };
            // if apple is on snake, regen apple
            for body in self.snake.iter() {
                if body.pos == apple_pos {
                    continue 'apple;
                }
            }
            self.apple = apple_pos;
            break;
        }
    }
    // returns false if snake cant move (game over)
    fn move_snake(&mut self) -> bool {
        {
            // move snake head
            let head = &mut self.snake[0];
            head.move_square();
            // check if snake ate apple
            if head.pos == self.apple {
                self.score += 1;
                println!("Score: {}", self.score);
                self.grow_snake();
                self.place_apple();
            }
            if self.snake.len() < MIN_SNAKE_LEN {
                self.grow_snake()
            }
            if self.snake_left_board() {
                println!("Snake left board");
                return false;
            }
        }

        let mut last = self.snake[0].direction;
        let head_pos = self.snake[0].pos;
        // move snake body
        for body in self.snake.iter_mut().skip(1) {
            body.move_square();
            let i = body.direction;
            body.direction = last;
            last = i;
            if head_pos == body.pos {
                println!("Snake collided with body");
                return false;
            }
        }
        true
    }
    // grows snake by one square
    fn grow_snake(&mut self) {
        // add square to snake
        self.snake.push(self.snake.last().unwrap().clone());

        let snake_len = self.snake.len();
        let snake_tail = self.snake.last_mut().unwrap();
        // move square to end of snake
        snake_tail
            .pos
            .move_direction(snake_tail.direction.opposite());
        // if snake only has head, repeat last action
        if snake_len == 2 {
            snake_tail
                .pos
                .move_direction(snake_tail.direction.opposite());
        }
    }
    // returns true if the snake left the board
    fn snake_left_board(&self) -> bool {
        let snake_head = self.snake[0];
        let board_size = self.settings.board_size;
        if snake_head.pos.x == board_size.x + 1
            || snake_head.pos.x == 0
            || snake_head.pos.y == board_size.y + 1
            || snake_head.pos.y == 0
        {
            return true;
        }
        false
    }
    // draws the game on screen, does no game logic itself
    fn draw_canvas(&self, canvas: &mut WindowCanvas) -> Result<(), String> {
        let win_size = canvas.viewport();
        let win_size = Vec2 {
            x: win_size.width(),
            y: win_size.height(),
        };
        let board_size = self.settings.board_size;

        // size of one square
        let mut square_size = (win_size.y - win_size.y / 8 - win_size.y / 20) / board_size.y;

        {
            // game board rectangle location, temporary value, recalculated later
            let xrect = win_size.x.checked_sub(board_size.x * square_size);
            let yrect = (win_size.y - win_size.y / 8).checked_sub(board_size.y * square_size);
            // check for overflow, if it does use different formula for square size
            if xrect.is_none() || yrect.is_none() {
                square_size = (win_size.x - win_size.x / 20) / board_size.x;
            }
        }

        // clear canvas
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();

        // draw rectangle game board is going to be on
        // serves as border for game board
        canvas.set_draw_color(Color::WHITE);
        canvas.fill_rect(Rect::new(
            0,
            win_size.y as i32 / 8,e
            win_size.x,
            win_size.y / 8 * 7,
        ))?;

        canvas.set_draw_color(Color::BLACK);
        // game board location
        let rect_pos = Vec2 {
            x: (win_size.x - board_size.x * square_size) / 2,
            y: ((win_size.y - win_size.y / 8) - board_size.y * square_size) / 2 + win_size.y / 8,
        };
        // draw game board
        canvas.fill_rect(Rect::new(
            rect_pos.x as i32,
            rect_pos.y as i32,
            board_size.x * square_size,
            board_size.y * square_size,
        ))?;

        let mut snake_rects = Vec::with_capacity(self.snake.len());
        // calculate the positions of the squares that make the snake
        for body in &self.snake {
            snake_rects.push(Rect::new(
                (body.pos.x * square_size + rect_pos.x - square_size) as i32,
                (body.pos.y * square_size + rect_pos.y - square_size) as i32,
                square_size,
                square_size,
            ));
        }
        // draw snake
        canvas.set_draw_color(Color::WHITE);
        canvas.fill_rects(&snake_rects)?;

        // draw apple
        canvas.set_draw_color(Color::RED);
        canvas.fill_rect(Rect::new(
            (self.apple.x * square_size + rect_pos.x - square_size) as i32,
            (self.apple.y * square_size + rect_pos.y - square_size) as i32,
            square_size,
            square_size,
        ))?;

        canvas.present();
        Ok(())
    }
}

#[derive(Copy, Clone)]
struct SnakeBody {
    // position
    pos: Vec2,
    direction: Direction,
}

impl SnakeBody {
    fn move_square(&mut self) {
        self.pos.move_direction(self.direction);
    }
}

// x is horisontal, y vertical
#[derive(Copy, Clone, PartialEq, Eq)]
struct Vec2 {
    x: u32,
    y: u32,
}

impl Vec2 {
    fn move_direction(&mut self, direction: Direction) {
        match direction {
            Direction::Up => self.y -= 1,
            Direction::Down => self.y += 1,
            Direction::Right => self.x += 1,
            Direction::Left => self.x -= 1,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn opposite(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}
