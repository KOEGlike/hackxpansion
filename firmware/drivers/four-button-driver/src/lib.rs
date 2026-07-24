#![no_std]

use xpanse_api::{
    bus::allocator::BusAllocator,
    driver::{Driver, DriverError, DriverMeta},
    gpio_bank::{BankPins, GpioBank},
    interfaces::buttons::{A, B, Down, Left, Right, Up, X, Y, aliased_pin_buttons},
    metadata::{ModuleDetectResistor, ModuleID, ModuleSlot},
    registry::{Registry, ResourceId},
};

pub struct FourButtonDriver;

impl DriverMeta for FourButtonDriver {
    const ID: ModuleID = ModuleID {
        md0: ModuleDetectResistor::R1K5,
        md1: ModuleDetectResistor::R1K6,
    };
}

impl<G: BankPins> Driver<G> for FourButtonDriver {
    async fn create(
        gpio_bank: GpioBank<G>,
        slot: ModuleSlot,
        registry: &mut Registry,
        bus_allocator: &mut BusAllocator,
    ) -> Result<(), DriverError> {
        let button_a_pin = gpio_bank.gpio1;
        let button_b_pin = gpio_bank.gpio0;
        let button_x_pin = gpio_bank.gpio2;
        let button_y_pin = gpio_bank.gpio3;

        let (button_a, button_down) = aliased_pin_buttons::<A, Down>(button_a_pin.into());
        let (button_b, button_right) = aliased_pin_buttons::<B, Right>(button_b_pin.into());
        let (button_x, button_left) = aliased_pin_buttons::<X, Left>(button_x_pin.into());
        let (button_y, button_up) = aliased_pin_buttons::<Y, Up>(button_y_pin.into());

        let button_a_id = ResourceId::new(slot, 0);
        let button_b_id = ResourceId::new(slot, 1);
        let button_x_id = ResourceId::new(slot, 2);
        let button_y_id = ResourceId::new(slot, 3);

        registry
            .register_with_id(slot, FourButtonDriver::ID, button_a_id, button_a)
            .map_err(|_| DriverError::InitFailed)?;
        registry
            .register_with_id(slot, FourButtonDriver::ID, button_a_id, button_down)
            .map_err(|_| DriverError::InitFailed)?;

        registry
            .register_with_id(slot, FourButtonDriver::ID, button_b_id, button_b)
            .map_err(|_| DriverError::InitFailed)?;
        registry
            .register_with_id(slot, FourButtonDriver::ID, button_b_id, button_right)
            .map_err(|_| DriverError::InitFailed)?;

        registry
            .register_with_id(slot, FourButtonDriver::ID, button_x_id, button_x)
            .map_err(|_| DriverError::InitFailed)?;
        registry
            .register_with_id(slot, FourButtonDriver::ID, button_x_id, button_left)
            .map_err(|_| DriverError::InitFailed)?;

        registry
            .register_with_id(slot, FourButtonDriver::ID, button_y_id, button_y)
            .map_err(|_| DriverError::InitFailed)?;
        registry
            .register_with_id(slot, FourButtonDriver::ID, button_y_id, button_up)
            .map_err(|_| DriverError::InitFailed)?;

        let _ = bus_allocator;

        Ok(())
    }
}
