use xpanse_driver_api::gpio_bank::GpioBank;

use crate::bus::spi_factory::PioManager;
use xpanse_driver_api::bus::allocator::BusAllocator;
use crate::load_driver;
use crate::{adc::init_adc, adc_mapping, resource_split::*};
use xpanse_driver_api::{driver::Driver, metadata::ModuleSlot};

slint::slint! {
    export component HelloWorld inherits Window {
        Text {
            text: "hello world";
            color: green;
        }
    }
}

#[embassy_executor::task]
pub async fn core1_task(
    gpio_bank_0: GpioBankPeris0,
    gpio_bank_1: GpioBankPeris1,
    gpio_bank_2: GpioBankPeris2,
    gpio_bank_3: GpioBankPeris3,
    mut i2c_pins: I2cPinPeris,
    mut remaining_peris: RemainingPeris,
) {
    let gpio_bank_0 = GpioBank::new(
        gpio_bank_0.gpio0,      //0
        gpio_bank_0.gpio1,      //1
        gpio_bank_0.gpio2,      //2
        gpio_bank_0.gpio3,      //3
        gpio_bank_0.gpio4,      //4
        gpio_bank_0.gpio5,      //5
        gpio_bank_0.gpio6,      //6
        gpio_bank_0.gpio7,      //7
        gpio_bank_0.gpio8,      //8
        gpio_bank_0.gpio9,      //9
        gpio_bank_0.pwm_slice1, //10
        gpio_bank_0.pwm_slice2, //11
        gpio_bank_0.pwm_slice3, //12
    );

    let gpio_bank_1 = GpioBank::new(
        gpio_bank_1.gpio0,      //0
        gpio_bank_1.gpio1,      //1
        gpio_bank_1.gpio2,      //2
        gpio_bank_1.gpio3,      //3
        gpio_bank_1.gpio4,      //4
        gpio_bank_1.gpio5,      //5
        gpio_bank_1.gpio6,      //6
        gpio_bank_1.gpio7,      //7
        gpio_bank_1.gpio8,      //8
        gpio_bank_1.gpio9,      //9
        gpio_bank_1.pwm_slice1, //10
        gpio_bank_1.pwm_slice2, //11
        gpio_bank_1.pwm_slice3, //12
    );

    let gpio_bank_2 = GpioBank::new(
        gpio_bank_2.gpio0,      //0
        gpio_bank_2.gpio1,      //1
        gpio_bank_2.gpio2,      //2
        gpio_bank_2.gpio3,      //3
        gpio_bank_2.gpio4,      //4
        gpio_bank_2.gpio5,      //5
        gpio_bank_2.gpio6,      //6
        gpio_bank_2.gpio7,      //7
        gpio_bank_2.gpio8,      //8
        gpio_bank_2.gpio9,      //9
        gpio_bank_2.pwm_slice1, //10
        gpio_bank_2.pwm_slice2, //11
        gpio_bank_2.pwm_slice3, //12
    );

    let gpio_bank_3 = GpioBank::new(
        gpio_bank_3.gpio0,      //0
        gpio_bank_3.gpio1,      //1
        gpio_bank_3.gpio2,      //2
        gpio_bank_3.gpio3,      //3
        gpio_bank_3.gpio4,      //4
        gpio_bank_3.gpio5,      //5
        gpio_bank_3.gpio6,      //6
        gpio_bank_3.gpio7,      //7
        gpio_bank_3.gpio8,      //8
        gpio_bank_3.gpio9,      //9
        gpio_bank_3.pwm_slice1, //10
        gpio_bank_3.pwm_slice2, //11
        gpio_bank_3.pwm_slice3, //12
    );

    let mut adc = init_adc(
        remaining_peris.i2c1.reborrow(),
        i2c_pins.sda.reborrow(),
        i2c_pins.scl.reborrow(),
    )
    .await
    .unwrap();

    let module_0_id = adc_mapping::map_adc(&mut adc, ModuleSlot::FrontRight)
        .await
        .unwrap();
    let module_1_id = adc_mapping::map_adc(&mut adc, ModuleSlot::FrontLeft)
        .await
        .unwrap();
    let module_2_id = adc_mapping::map_adc(&mut adc, ModuleSlot::BackRight)
        .await
        .unwrap();
    let module_3_id = adc_mapping::map_adc(&mut adc, ModuleSlot::BackLeft)
        .await
        .unwrap();

    let mut _pio_manager = PioManager::new(
        remaining_peris.pio0,
        remaining_peris.pio1,
        remaining_peris.pio2,
    );

    let mut bus_allocator = BusAllocator::new(
        remaining_peris.spi1,
        remaining_peris.i2c0,
        remaining_peris.i2c1,
        remaining_peris.uart0,
        remaining_peris.uart1,
    );

    let mut registry = crate::device_registry::DeviceRegistry::new();

    load_driver!(
        module_0_id,
        gpio_bank_0,
        ModuleSlot::FrontRight,
        &mut registry,
        &mut bus_allocator
    );
    load_driver!(
        module_1_id,
        gpio_bank_1,
        ModuleSlot::FrontLeft,
        &mut registry,
        &mut bus_allocator
    );
    load_driver!(
        module_2_id,
        gpio_bank_2,
        ModuleSlot::BackRight,
        &mut registry,
        &mut bus_allocator
    );
    load_driver!(
        module_3_id,
        gpio_bank_3,
        ModuleSlot::BackLeft,
        &mut registry,
        &mut bus_allocator
    );
}
