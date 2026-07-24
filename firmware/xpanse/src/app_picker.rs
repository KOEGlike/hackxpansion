extern crate alloc;

use alloc::{boxed::Box, rc::Rc, vec::Vec};

use embassy_futures::select::{Either5, select5};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use xpanse_api::{
    interfaces::buttons::{A, B, Button, ButtonRole, Down, Up},
    registry::{RegisteredResource, Registry, ResourceId},
};
slint::include_modules!();

use crate::app_loader::{self, AppDescriptor};

static APP_PICKER_INPUT: Channel<CriticalSectionRawMutex, AppPickerInput, 8> = Channel::new();

#[derive(Clone, Copy)]
enum AppPickerInput {
    Up,
    Down,
    Left,
    Right,
    Cycle,
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

pub(crate) fn create_app_picker() -> Result<AppPickerUI, slint::PlatformError> {
    AppPickerUI::new()
}

pub(crate) async fn pick_app(
    registry: &mut Registry,
    app_picker: &AppPickerUI,
) -> &'static AppDescriptor {
    loop {
        clear_app_picker_input();
        attach_app_picker(app_picker);

        let app_count = app_loader::runnable_app_count(registry);
        let mut selected_index = (app_count > 0).then_some(0);

        app_picker.set_apps(runnable_app_names(registry));
        set_picker_selected_index(app_picker, selected_index);
        if app_picker.show().is_err() {
            defmt::error!("app picker: failed to show UI");
            core::future::pending::<()>().await;
        }

        let (mut select_button, mut cycle_button, mut up_button, mut down_button) =
            take_picker_buttons(registry);

        let selected_index = loop {
            match receive_app_picker_input(
                select_button.as_mut(),
                cycle_button.as_mut(),
                up_button.as_mut(),
                down_button.as_mut(),
            )
            .await
            {
                AppPickerInput::Up | AppPickerInput::Left => {
                    move_picker_selection(
                        &mut selected_index,
                        app_count,
                        SelectionDirection::Previous,
                    );
                    set_picker_selected_index(app_picker, selected_index);
                }
                AppPickerInput::Down | AppPickerInput::Right | AppPickerInput::Cycle => {
                    move_picker_selection(&mut selected_index, app_count, SelectionDirection::Next);
                    set_picker_selected_index(app_picker, selected_index);
                }
                AppPickerInput::Select => {
                    if let Some(selected_index) = selected_index {
                        break selected_index;
                    }
                }
            }
        };

        if app_picker.hide().is_err() {
            defmt::error!("app picker: failed to hide UI");
        }
        if let Some(select_button) = select_button {
            registry.return_resource(select_button);
        }
        if let Some(cycle_button) = cycle_button {
            registry.return_resource(cycle_button);
        }
        if let Some(up_button) = up_button {
            registry.return_resource(up_button);
        }
        if let Some(down_button) = down_button {
            registry.return_resource(down_button);
        }

        if let Some(app) = app_loader::runnable_app_at(registry, selected_index) {
            return app;
        }

        defmt::warn!("app picker returned an invalid selection");
    }
}

fn attach_app_picker(app_picker: &AppPickerUI) {
    use slint::private_unstable_api::re_exports::{VRc, WindowInner};

    let component = VRc::into_dyn(app_picker.clone_strong().into());
    WindowInner::from_pub(app_picker.window()).set_component(&component);
}

async fn receive_app_picker_input(
    select_button: Option<&mut RegisteredResource<Box<dyn Button<A>>>>,
    cycle_button: Option<&mut RegisteredResource<Box<dyn Button<B>>>>,
    up_button: Option<&mut RegisteredResource<Box<dyn Button<Up>>>>,
    down_button: Option<&mut RegisteredResource<Box<dyn Button<Down>>>>,
) -> AppPickerInput {
    match select5(
        wait_for_picker_button(select_button, AppPickerInput::Select),
        wait_for_picker_button(cycle_button, AppPickerInput::Cycle),
        wait_for_picker_button(up_button, AppPickerInput::Up),
        wait_for_picker_button(down_button, AppPickerInput::Down),
        APP_PICKER_INPUT.receive(),
    )
    .await
    {
        Either5::First(input)
        | Either5::Second(input)
        | Either5::Third(input)
        | Either5::Fourth(input)
        | Either5::Fifth(input) => input,
    }
}

type PickerButton<R> = Option<RegisteredResource<Box<dyn Button<R>>>>;

async fn wait_for_picker_button<R: ButtonRole>(
    button: Option<&mut RegisteredResource<Box<dyn Button<R>>>>,
    input: AppPickerInput,
) -> AppPickerInput {
    match button {
        Some(button) => {
            button.resource.wait_for_pressed().await;
            input
        }
        None => core::future::pending().await,
    }
}

fn take_picker_buttons(
    registry: &mut Registry,
) -> (
    PickerButton<A>,
    PickerButton<B>,
    PickerButton<Up>,
    PickerButton<Down>,
) {
    let mut ids = Vec::new();
    let select = take_button_with_distinct_id(registry, &mut ids);
    let cycle = take_button_with_distinct_id(registry, &mut ids);
    let up = take_button_with_distinct_id(registry, &mut ids);
    let down = take_button_with_distinct_id(registry, &mut ids);

    (select, cycle, up, down)
}

fn take_button_with_distinct_id<R: ButtonRole>(
    registry: &mut Registry,
    ids: &mut Vec<ResourceId>,
) -> PickerButton<R> {
    let mut aliases = Vec::new();

    let button = loop {
        match registry.take_resource::<Box<dyn Button<R>>>() {
            Some(button) if !ids.contains(&button.id()) => break Some(button),
            Some(alias) => aliases.push(alias),
            None => break None,
        }
    };

    for alias in aliases {
        registry.return_resource(alias);
    }
    if let Some(button) = &button {
        ids.push(button.id());
    }

    button
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
