extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;
use xpanse_driver_api::interfaces::buttons::Button;
use xpanse_driver_api::metadata::{ModuleID, ModuleSlot};
use xpanse_driver_api::registry::{Register, RegisteredResource};

#[derive(Default)]
pub struct DeviceRegistry {
    pub buttons: Vec<RegisteredResource<dyn Button>>,
    // Add other capability lists here as you define new traits!
}

impl DeviceRegistry {
    pub fn new() -> Self {
        Self {
            buttons: Vec::new(),
        }
    }
}

// We prove to the API that our registry knows how to accept Buttons
impl Register<dyn Button> for DeviceRegistry {
    fn register(&mut self, slot: ModuleSlot, module_id: ModuleID, item: Box<dyn Button>) {
        self.buttons.push(RegisteredResource {
            slot,
            module_id,
            resource: item,
        });
    }
}
