extern crate alloc;

use alloc::{rc::Rc, vec::Vec};

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use xpanse_driver_api::registry::Registry;
slint::include_modules!();

use crate::app_loader::{self, AppDescriptor};

static APP_PICKER_INPUT: Channel<CriticalSectionRawMutex, AppPickerInput, 8> = Channel::new();

#[derive(Clone, Copy)]
enum AppPickerInput {
    Up,
    Down,
    Left,
    Right,
    Select,
}

pub fn up() -> bool {
    send_app_picker_input(AppPickerInput::Up)
}

pub fn down() -> bool {
    send_app_picker_input(AppPickerInput::Down)
}

pub fn left() -> bool {
    send_app_picker_input(AppPickerInput::Left)
}

pub fn right() -> bool {
    send_app_picker_input(AppPickerInput::Right)
}

pub fn select() -> bool {
    send_app_picker_input(AppPickerInput::Select)
}

fn send_app_picker_input(input: AppPickerInput) -> bool {
    APP_PICKER_INPUT.try_send(input).is_ok()
}

pub(crate) async fn pick_app(registry: &Registry) -> &'static AppDescriptor {
    loop {
        clear_app_picker_input();

        let app_count = app_loader::runnable_app_count(registry);
        let mut selected_index = (app_count > 0).then_some(0);

        let app_picker = AppPickerUI::new().unwrap();
        app_picker.set_apps(runnable_app_names(registry));
        set_picker_selected_index(&app_picker, selected_index);
        app_picker.show().unwrap();

        let selected_index = loop {
            match APP_PICKER_INPUT.receive().await {
                AppPickerInput::Up | AppPickerInput::Left => {
                    move_picker_selection(
                        &mut selected_index,
                        app_count,
                        SelectionDirection::Previous,
                    );
                    set_picker_selected_index(&app_picker, selected_index);
                }
                AppPickerInput::Down | AppPickerInput::Right => {
                    move_picker_selection(&mut selected_index, app_count, SelectionDirection::Next);
                    set_picker_selected_index(&app_picker, selected_index);
                }
                AppPickerInput::Select => {
                    if let Some(selected_index) = selected_index {
                        break selected_index;
                    }
                }
            }
        };

        app_picker.hide().unwrap();

        if let Some(app) = app_loader::runnable_app_at(registry, selected_index) {
            return app;
        }

        defmt::warn!("app picker returned an invalid selection");
    }
}

fn clear_app_picker_input() {
    while APP_PICKER_INPUT.try_receive().is_ok() {}
}

fn runnable_app_names(registry: &Registry) -> ModelRc<SharedString> {
    let names = app_loader::runnable_apps(registry)
        .map(|app| SharedString::from(app.name))
        .collect::<Vec<_>>();

    ModelRc::from(Rc::new(VecModel::from(names)))
}

enum SelectionDirection {
    Previous,
    Next,
}

fn move_picker_selection(
    selected_index: &mut Option<usize>,
    app_count: usize,
    direction: SelectionDirection,
) {
    if app_count == 0 {
        *selected_index = None;
        return;
    }

    let next_index = match (*selected_index, direction) {
        // the two none cases are only for safety, they could be replaced with _ => 0
        (None, SelectionDirection::Previous) => app_count - 1,
        (None, SelectionDirection::Next) => 0,
        (Some(0), SelectionDirection::Previous) => app_count - 1,
        (Some(index), SelectionDirection::Previous) => index - 1,
        (Some(index), SelectionDirection::Next) => (index + 1) % app_count,
    };

    *selected_index = Some(next_index);
}

fn set_picker_selected_index(app_picker: &AppPickerUI, selected_index: Option<usize>) {
    app_picker.set_selected_index(selected_index.map(|index| index as i32).unwrap_or(-1));
}
