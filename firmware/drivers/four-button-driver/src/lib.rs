#![no_std]

use xpanse_api::{
    bus::allocator::BusAllocator,
    driver::{Driver, DriverError, DriverMeta},
    gpio_bank::{BankPins, GpioBank},
    interfaces::buttons::{A, B, Down, Left, Right, Up, X, Y, aliased_pin_buttons},
    metadata::{ModuleDetectResistor, ModuleID, ModuleSlot},
    registry::Registry,
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

        registry
            .register_groups(
                slot,
                FourButtonDriver::ID,
                (
                    (button_a, button_down),
                    (button_b, button_right),
                    (button_x, button_left),
                    (button_y, button_up),
                ),
            )
            .map_err(|_| DriverError::InitFailed)?;

        let _ = bus_allocator;

        Ok(())
    }
}
