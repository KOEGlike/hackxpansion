#![no_std]

extern crate alloc;

use alloc::{boxed::Box, rc::Rc, vec::Vec};
use core::{future::Future, pin::Pin};

use embassy_futures::select::{Either4, Either5, select4, select5};
use embassy_time::{Duration, Ticker};
use heapless::Vec as FixedVec;
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use xpanse_api::{
    app::App,
    interfaces::buttons::{Button, Down, Left, Right, Up},
    registry::{Registry, ResourceLease},
};

slint::include_modules!();

const LEVEL_TICK_INTERVALS: [Duration; 3] = [
    Duration::from_millis(80),
    Duration::from_millis(32),
    Duration::from_millis(25),
];
const LEVEL_BPMS: [i32; 3] = [75, 94, 120];
const SPAWN_INTERVAL_TICKS: u32 = 10;
const SPAWN_Y: i16 = 38;
const TARGET_Y: i16 = 179;
const NOTE_SPEED: i16 = 3;
const PERFECT_WINDOW: i16 = 5;
const GOOD_WINDOW: i16 = 13;
const MAX_NOTES: usize = 8;
const STARTING_HEALTH: u8 = 100;
const GHOST_TAP_DAMAGE: u8 = 6;
const MISSED_NOTE_DAMAGE: u8 = 12;
const FLASH_TICKS: u8 = 4;

type AppButton<R> = ResourceLease<Box<dyn Button<R>>>;
type RhythmControls = (
    Box<dyn Button<Up>>,
    Box<dyn Button<Down>>,
    Box<dyn Button<Left>>,
    Box<dyn Button<Right>>,
);

pub struct NeonBeatApp {
    up: AppButton<Up>,
    down: AppButton<Down>,
    left: AppButton<Left>,
    right: AppButton<Right>,
}

impl App for NeonBeatApp {
    const NAME: &'static str = "Neon Beat";

    fn can_run(registry: &Registry) -> bool {
        registry.has_resource_set::<RhythmControls>()
    }

    fn new(registry: &mut Registry) -> Option<Self> {
        let (up, down, left, right) = registry.take_resource_set::<RhythmControls>()?;
        Some(Self {
            up,
            down,
            left,
            right,
        })
    }

    fn run<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
        Box::pin(async move {
            let ui = match NeonBeatUI::new() {
                Ok(ui) => ui,
                Err(_) => {
                    defmt::error!("NeonBeatApp: failed to create UI");
                    return;
                }
            };

            let mut game = Game::new();
            let note_model = Rc::new(VecModel::from(game.ui_notes()));
            ui.set_notes(ModelRc::from(note_model.clone()));
            update_stats(&ui, &game);
            ui.set_level(3);
            ui.set_bpm(LEVEL_BPMS[2]);

            if ui.show().is_err() {
                defmt::error!("NeonBeatApp: failed to show UI");
                return;
            }

            let Some(level_index) = ({
                let mut level_index = 2usize;
                'select: loop {
                    match select4(
                        self.up.resource_mut().wait_for_pressed(),
                        self.down.resource_mut().wait_for_pressed(),
                        self.left.resource_mut().wait_for_pressed(),
                        self.right.resource_mut().wait_for_pressed(),
                    )
                    .await
                    {
                        Either4::First(()) => level_index = level_index.saturating_sub(1),
                        Either4::Second(()) => level_index = (level_index + 1).min(2),
                        Either4::Third(()) => break 'select None,
                        Either4::Fourth(()) => break 'select Some(level_index),
                    }
                    ui.set_level(level_index as i32 + 1);
                    ui.set_bpm(LEVEL_BPMS[level_index]);
                }
            }) else {
                if ui.hide().is_err() {
                    defmt::error!("NeonBeatApp: failed to hide UI");
                }
                return;
            };
            ui.set_selecting_level(false);

            let game_over = {
                let up = self.up.resource_mut();
                let down = self.down.resource_mut();
                let left = self.left.resource_mut();
                let right = self.right.resource_mut();
                let mut ticker = Ticker::every(LEVEL_TICK_INTERVALS[level_index]);
                let mut up_pressed = up.wait_for_pressed();
                let mut down_pressed = down.wait_for_pressed();
                let mut left_pressed = left.wait_for_pressed();
                let mut right_pressed = right.wait_for_pressed();
                let mut tick = ticker.next();

                'game: loop {
                    let pressed_lane = match select5(
                        &mut up_pressed,
                        &mut down_pressed,
                        &mut left_pressed,
                        &mut right_pressed,
                        &mut tick,
                    )
                    .await
                    {
                        Either5::First(()) => Some(Lane::Up),
                        Either5::Second(()) => Some(Lane::Down),
                        Either5::Third(()) => Some(Lane::Left),
                        Either5::Fourth(()) => Some(Lane::Right),
                        Either5::Fifth(()) => {
                            drop(tick);
                            let missed_note = game.advance();
                            sync_note_model(&note_model, &game);
                            ui.set_active_lane(game.active_lane_index());
                            if missed_note {
                                update_stats(&ui, &game);
                            }
                            if game.is_over() {
                                ui.set_game_over(true);
                                break 'game true;
                            }
                            tick = ticker.next();
                            None
                        }
                    };

                    let Some(lane) = pressed_lane else {
                        continue;
                    };

                    drop(up_pressed);
                    drop(down_pressed);
                    drop(left_pressed);
                    drop(right_pressed);

                    if up.is_pressed() && down.is_pressed() {
                        break 'game false;
                    }

                    game.press(lane);
                    sync_note_model(&note_model, &game);
                    update_stats(&ui, &game);
                    ui.set_active_lane(game.active_lane_index());
                    if game.is_over() {
                        ui.set_game_over(true);
                        break 'game true;
                    }

                    up_pressed = up.wait_for_pressed();
                    down_pressed = down.wait_for_pressed();
                    left_pressed = left.wait_for_pressed();
                    right_pressed = right.wait_for_pressed();
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
                defmt::error!("NeonBeatApp: failed to hide UI");
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
enum Lane {
    Left,
    Down,
    Up,
    Right,
}

impl Lane {
    const fn index(self) -> i32 {
        match self {
            Self::Left => 0,
            Self::Down => 1,
            Self::Up => 2,
            Self::Right => 3,
        }
    }
}

const CHART: [Lane; 32] = [
    Lane::Left,
    Lane::Down,
    Lane::Up,
    Lane::Right,
    Lane::Left,
    Lane::Up,
    Lane::Down,
    Lane::Right,
    Lane::Left,
    Lane::Right,
    Lane::Down,
    Lane::Up,
    Lane::Down,
    Lane::Left,
    Lane::Right,
    Lane::Up,
    Lane::Left,
    Lane::Down,
    Lane::Left,
    Lane::Right,
    Lane::Up,
    Lane::Right,
    Lane::Down,
    Lane::Up,
    Lane::Right,
    Lane::Left,
    Lane::Up,
    Lane::Down,
    Lane::Left,
    Lane::Right,
    Lane::Up,
    Lane::Down,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FallingNote {
    lane: Lane,
    y: i16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Judgement {
    Ready,
    Perfect,
    Good,
    Miss,
}

impl Judgement {
    const fn label(self) -> &'static str {
        match self {
            Self::Ready => "GET READY",
            Self::Perfect => "PERFECT!",
            Self::Good => "GOOD",
            Self::Miss => "MISS",
        }
    }

    const fn kind(self) -> i32 {
        match self {
            Self::Ready => 0,
            Self::Perfect => 1,
            Self::Good => 2,
            Self::Miss => 3,
        }
    }
}

struct Game {
    notes: FixedVec<FallingNote, MAX_NOTES>,
    chart_index: usize,
    tick: u32,
    score: u32,
    combo: u16,
    health: u8,
    judgement: Judgement,
    active_lane: Option<Lane>,
    flash_ticks: u8,
}

impl Game {
    fn new() -> Self {
        let mut game = Self {
            notes: FixedVec::new(),
            chart_index: 0,
            tick: 0,
            score: 0,
            combo: 0,
            health: STARTING_HEALTH,
            judgement: Judgement::Ready,
            active_lane: None,
            flash_ticks: 0,
        };
        game.spawn_note();
        game
    }

    fn press(&mut self, lane: Lane) {
        self.active_lane = Some(lane);
        self.flash_ticks = FLASH_TICKS;

        let closest = self
            .notes
            .iter()
            .enumerate()
            .filter(|(_, note)| note.lane == lane)
            .map(|(index, note)| (index, (note.y - TARGET_Y).abs()))
            .min_by_key(|(_, distance)| *distance);

        let Some((index, distance)) = closest else {
            self.apply_miss(GHOST_TAP_DAMAGE);
            return;
        };
        if distance > GOOD_WINDOW {
            self.apply_miss(GHOST_TAP_DAMAGE);
            return;
        }

        self.notes.remove(index);
        self.combo = self.combo.saturating_add(1);
        let combo_bonus = u32::from(self.combo) * 10;
        if distance <= PERFECT_WINDOW {
            self.score = self.score.saturating_add(1_000 + combo_bonus);
            self.health = self.health.saturating_add(2).min(STARTING_HEALTH);
            self.judgement = Judgement::Perfect;
        } else {
            self.score = self.score.saturating_add(500 + combo_bonus);
            self.health = self.health.saturating_add(1).min(STARTING_HEALTH);
            self.judgement = Judgement::Good;
        }
    }

    fn advance(&mut self) -> bool {
        self.tick = self.tick.wrapping_add(1);
        if self.flash_ticks > 0 {
            self.flash_ticks -= 1;
            if self.flash_ticks == 0 {
                self.active_lane = None;
            }
        }

        for note in &mut self.notes {
            note.y += NOTE_SPEED;
        }

        let mut missed_note = false;
        let mut index = 0;
        while index < self.notes.len() {
            if self.notes[index].y > TARGET_Y + GOOD_WINDOW {
                self.notes.remove(index);
                self.apply_miss(MISSED_NOTE_DAMAGE);
                missed_note = true;
            } else {
                index += 1;
            }
        }

        if self.tick.is_multiple_of(SPAWN_INTERVAL_TICKS) && !self.is_over() {
            self.spawn_note();
        }

        missed_note
    }

    fn spawn_note(&mut self) {
        let lane = CHART[self.chart_index];
        self.chart_index = (self.chart_index + 1) % CHART.len();
        let _ = self.notes.push(FallingNote { lane, y: SPAWN_Y });
    }

    fn apply_miss(&mut self, damage: u8) {
        self.combo = 0;
        self.health = self.health.saturating_sub(damage);
        self.judgement = Judgement::Miss;
    }

    const fn is_over(&self) -> bool {
        self.health == 0
    }

    fn active_lane_index(&self) -> i32 {
        self.active_lane.map(Lane::index).unwrap_or(-1)
    }

    fn ui_notes(&self) -> Vec<BeatNote> {
        self.notes
            .iter()
            .map(|note| BeatNote {
                lane: note.lane.index(),
                y: i32::from(note.y),
            })
            .collect()
    }
}

fn sync_note_model(model: &VecModel<BeatNote>, game: &Game) {
    while model.row_count() > game.notes.len() {
        model.remove(model.row_count() - 1);
    }

    for (index, note) in game.notes.iter().enumerate() {
        let ui_note = BeatNote {
            lane: note.lane.index(),
            y: i32::from(note.y),
        };
        if index < model.row_count() {
            model.set_row_data(index, ui_note);
        } else {
            model.push(ui_note);
        }
    }
}

fn update_stats(ui: &NeonBeatUI, game: &Game) {
    ui.set_score(game.score.min(i32::MAX as u32) as i32);
    ui.set_combo(i32::from(game.combo));
    ui.set_health(i32::from(game.health));
    ui.set_judgement(SharedString::from(game.judgement.label()));
    ui.set_judgement_kind(game.judgement.kind());
}
