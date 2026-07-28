#![no_std]

extern crate alloc;

use alloc::{boxed::Box, rc::Rc, vec::Vec};
use core::{future::Future, pin::Pin};

use embassy_futures::select::{Either5, select4, select5};
use embassy_time::{Duration, Instant, Ticker};
use heapless::{Deque, Vec as FixedVec};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use xpanse_api::{
    app::App,
    interfaces::buttons::{Button, Down, Left, Right, Up},
    registry::{Registry, ResourceLease},
};

slint::include_modules!();

const BOARD_WIDTH: u8 = 20;
const BOARD_HEIGHT: u8 = 13;
const BOARD_CELLS: usize = BOARD_WIDTH as usize * BOARD_HEIGHT as usize;
const TICK_INTERVAL: Duration = Duration::from_millis(140);

type AppButton<R> = ResourceLease<Box<dyn Button<R>>>;
type SnakeControls = (
    Box<dyn Button<Up>>,
    Box<dyn Button<Down>>,
    Box<dyn Button<Left>>,
    Box<dyn Button<Right>>,
);

pub struct SnakeGameApp {
    up: AppButton<Up>,
    down: AppButton<Down>,
    left: AppButton<Left>,
    right: AppButton<Right>,
}

impl App for SnakeGameApp {
    const NAME: &'static str = "Snake";

    fn can_run(registry: &Registry) -> bool {
        registry.has_resource_set::<SnakeControls>()
    }

    fn new(registry: &mut Registry) -> Option<Self> {
        let (up, down, left, right) = registry.take_resource_set::<SnakeControls>()?;
        Some(Self {
            up,
            down,
            left,
            right,
        })
    }

    fn run<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
        Box::pin(async move {
            let ui = match SnakeGameUI::new() {
                Ok(ui) => ui,
                Err(_) => {
                    defmt::error!("SnakeGameApp: failed to create UI");
                    return;
                }
            };

            let mut game = Game::new(Instant::now().as_ticks() as u32);
            let snake_model = Rc::new(VecModel::from(game.ui_cells()));
            ui.set_snake_cells(ModelRc::from(snake_model.clone()));
            update_ui(&ui, &game);

            if ui.show().is_err() {
                defmt::error!("SnakeGameApp: failed to show UI");
                return;
            }

            let game_over = {
                let up = self.up.resource_mut();
                let down = self.down.resource_mut();
                let left = self.left.resource_mut();
                let right = self.right.resource_mut();
                let mut ticker = Ticker::every(TICK_INTERVAL);
                let mut up_pressed = up.wait_for_pressed();
                let mut down_pressed = down.wait_for_pressed();
                let mut left_pressed = left.wait_for_pressed();
                let mut right_pressed = right.wait_for_pressed();
                let mut tick = ticker.next();

                'game: loop {
                    let input = select5(
                        &mut up_pressed,
                        &mut down_pressed,
                        &mut left_pressed,
                        &mut right_pressed,
                        &mut tick,
                    )
                    .await;

                    match input {
                        Either5::First(()) => {
                            drop(up_pressed);
                            drop(down_pressed);
                            if up.is_pressed() && down.is_pressed() {
                                break 'game false;
                            }
                            game.queue_direction(Direction::Up);
                            up_pressed = up.wait_for_pressed();
                            down_pressed = down.wait_for_pressed();
                        }
                        Either5::Second(()) => {
                            drop(up_pressed);
                            drop(down_pressed);
                            if up.is_pressed() && down.is_pressed() {
                                break 'game false;
                            }
                            game.queue_direction(Direction::Down);
                            up_pressed = up.wait_for_pressed();
                            down_pressed = down.wait_for_pressed();
                        }
                        Either5::Third(()) => {
                            drop(left_pressed);
                            game.queue_direction(Direction::Left);
                            left_pressed = left.wait_for_pressed();
                        }
                        Either5::Fourth(()) => {
                            drop(right_pressed);
                            game.queue_direction(Direction::Right);
                            right_pressed = right.wait_for_pressed();
                        }
                        Either5::Fifth(()) => {
                            drop(tick);
                            match game.advance() {
                                AdvanceResult::Moved => sync_snake_model(&snake_model, &game),
                                AdvanceResult::Ate => {
                                    sync_snake_model(&snake_model, &game);
                                    update_ui(&ui, &game);
                                }
                                AdvanceResult::GameOver => {
                                    ui.set_game_over(true);
                                    break 'game true;
                                }
                                AdvanceResult::Won => {
                                    sync_snake_model(&snake_model, &game);
                                    update_ui(&ui, &game);
                                    ui.set_won(true);
                                    ui.set_game_over(true);
                                    break 'game true;
                                }
                            }
                            tick = ticker.next();
                        }
                    }
                }
            };

            if game_over {
                let _ = select4(
                    self.up.resource_mut().wait_for_pressed(),
                    self.down.resource_mut().wait_for_pressed(),
                    self.left.resource_mut().wait_for_pressed(),
                    self.right.resource_mut().wait_for_pressed(),
                )
                .await;
            }

            if ui.hide().is_err() {
                defmt::error!("SnakeGameApp: failed to hide UI");
            }
        })
    }

    fn release(self, registry: &mut Registry) {
        registry.return_resource(self.up);
        registry.return_resource(self.down);
        registry.return_resource(self.left);
        registry.return_resource(self.right);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Cell {
    x: u8,
    y: u8,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    const fn is_opposite(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Up, Self::Down)
                | (Self::Down, Self::Up)
                | (Self::Left, Self::Right)
                | (Self::Right, Self::Left)
        )
    }
}

enum AdvanceResult {
    Moved,
    Ate,
    GameOver,
    Won,
}

struct Game {
    snake: FixedVec<Cell, BOARD_CELLS>,
    direction: Direction,
    queued_directions: Deque<Direction, 2>,
    food: Option<Cell>,
    score: u16,
    rng: XorShift32,
}

impl Game {
    fn new(seed: u32) -> Self {
        let mut snake = FixedVec::new();
        snake.push(Cell { x: 10, y: 6 }).unwrap();
        snake.push(Cell { x: 9, y: 6 }).unwrap();
        snake.push(Cell { x: 8, y: 6 }).unwrap();
        snake.push(Cell { x: 7, y: 6 }).unwrap();

        let mut game = Self {
            snake,
            direction: Direction::Right,
            queued_directions: Deque::new(),
            food: None,
            score: 0,
            rng: XorShift32::new(seed),
        };
        game.food = game.place_food();
        game
    }

    fn queue_direction(&mut self, direction: Direction) {
        let previous = self
            .queued_directions
            .back()
            .copied()
            .unwrap_or(self.direction);
        if direction != previous && !direction.is_opposite(previous) {
            let _ = self.queued_directions.push_back(direction);
        }
    }

    fn advance(&mut self) -> AdvanceResult {
        if let Some(direction) = self.queued_directions.pop_front() {
            self.direction = direction;
        }
        let head = self.snake[0];
        let (dx, dy) = match self.direction {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        };
        let next_x = i16::from(head.x) + dx;
        let next_y = i16::from(head.y) + dy;
        if next_x < 0
            || next_x >= i16::from(BOARD_WIDTH)
            || next_y < 0
            || next_y >= i16::from(BOARD_HEIGHT)
        {
            return AdvanceResult::GameOver;
        }

        let next = Cell {
            x: next_x as u8,
            y: next_y as u8,
        };
        let ate = self.food == Some(next);
        let collision_length = self.snake.len() - usize::from(!ate);
        if self.snake[..collision_length].contains(&next) {
            return AdvanceResult::GameOver;
        }

        if !ate {
            self.snake.pop();
        }
        self.snake
            .insert(0, next)
            .expect("snake has room for its next cell");

        if !ate {
            return AdvanceResult::Moved;
        }

        self.score += 1;
        self.food = self.place_food();
        if self.food.is_some() {
            AdvanceResult::Ate
        } else {
            AdvanceResult::Won
        }
    }

    fn place_food(&mut self) -> Option<Cell> {
        let free_cells = BOARD_CELLS.checked_sub(self.snake.len())?;
        if free_cells == 0 {
            return None;
        }

        let mut target = self.rng.next() as usize % free_cells;
        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                let cell = Cell { x, y };
                if self.snake.contains(&cell) {
                    continue;
                }
                if target == 0 {
                    return Some(cell);
                }
                target -= 1;
            }
        }
        None
    }

    fn ui_cells(&self) -> Vec<SnakeCell> {
        self.snake.iter().copied().map(SnakeCell::from).collect()
    }
}

struct XorShift32(u32);

impl XorShift32 {
    fn new(seed: u32) -> Self {
        Self(if seed == 0 { 0x6d2b_79f5 } else { seed })
    }

    fn next(&mut self) -> u32 {
        let mut value = self.0;
        value ^= value << 13;
        value ^= value >> 17;
        value ^= value << 5;
        self.0 = value;
        value
    }
}

impl From<Cell> for SnakeCell {
    fn from(cell: Cell) -> Self {
        Self {
            x: i32::from(cell.x),
            y: i32::from(cell.y),
        }
    }
}

fn sync_snake_model(model: &VecModel<SnakeCell>, game: &Game) {
    model.insert(0, SnakeCell::from(game.snake[0]));
    while model.row_count() > game.snake.len() {
        model.remove(model.row_count() - 1);
    }
}

fn update_ui(ui: &SnakeGameUI, game: &Game) {
    ui.set_score(i32::from(game.score));
    if let Some(food) = game.food {
        ui.set_food_x(i32::from(food.x));
        ui.set_food_y(i32::from(food.y));
    } else {
        ui.set_food_x(-1);
        ui.set_food_y(-1);
    }
}
