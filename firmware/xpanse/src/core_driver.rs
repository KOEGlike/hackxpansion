use xpanse_driver_api::gpio_bank::GpioBank;
use xpanse_driver_api::registry::Registry;

use crate::load_driver::load_driver;
use crate::{adc::init_adc, adc_mapping, resource_split::*};
use xpanse_driver_api::bus::allocator::BusAllocator;
use xpanse_driver_api::interfaces::adc;
use xpanse_driver_api::metadata::ModuleSlot;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

macro_rules! gpio_bank_from_peris {
    ($peri:expr) => {
        GpioBank::new(
            $peri.gpio0,
            $peri.gpio1,
            $peri.gpio2,
            $peri.gpio3,
            $peri.gpio4,
            $peri.gpio5,
            $peri.gpio6,
            $peri.gpio7,
            $peri.gpio8,
            $peri.gpio9,
            $peri.pwm_slice1,
            $peri.pwm_slice2,
            $peri.pwm_slice3,
        )
    };
}

static REGISTRY_HANDOFF: Signal<CriticalSectionRawMutex, Registry> = Signal::new();

pub async fn take_registry() -> Registry {
    REGISTRY_HANDOFF.wait().await
}

#[embassy_executor::task]
pub async fn app_core_task(
    gpio_bank_0: GpioBankPeris0,
    gpio_bank_1: GpioBankPeris1,
    gpio_bank_2: GpioBankPeris2,
    gpio_bank_3: GpioBankPeris3,
    mut i2c_pins: I2cPinPeris,
    mut remaining_peris: RemainingPeris,
) {
    let gpio_bank_0 = gpio_bank_from_peris!(gpio_bank_0);
    let gpio_bank_1 = gpio_bank_from_peris!(gpio_bank_1);
    let gpio_bank_2 = gpio_bank_from_peris!(gpio_bank_2);
    let gpio_bank_3 = gpio_bank_from_peris!(gpio_bank_3);

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

    adc::init_adc(remaining_peris.adc, remaining_peris.adc_temp);

    let mut bus_allocator = BusAllocator::new(
        None,
        Some(remaining_peris.spi1),
        Some(remaining_peris.i2c0),
        Some(remaining_peris.i2c1),
        Some(remaining_peris.uart0),
        Some(remaining_peris.uart1),
        xpanse_driver_api::bus::allocator::DmaPool {
            ch0: Some(remaining_peris.dma_ch0),
            ch1: Some(remaining_peris.dma_ch1),
            ch2: Some(remaining_peris.dma_ch2),
            ch3: Some(remaining_peris.dma_ch3),
            ch4: Some(remaining_peris.dma_ch4),
            ch5: Some(remaining_peris.dma_ch5),
            ch6: Some(remaining_peris.dma_ch6),
            ch7: Some(remaining_peris.dma_ch7),
            ch8: Some(remaining_peris.dma_ch8),
            ch9: Some(remaining_peris.dma_ch9),
            ch10: Some(remaining_peris.dma_ch10),
            ch11: Some(remaining_peris.dma_ch11),
            ch12: Some(remaining_peris.dma_ch12),
            ch13: Some(remaining_peris.dma_ch13),
            ch14: Some(remaining_peris.dma_ch14),
            ch15: Some(remaining_peris.dma_ch15),
        },
        remaining_peris.pio0,
        remaining_peris.pio1,
        remaining_peris.pio2,
    );

    let mut registry = Registry::new();

    load_driver(
        module_0_id,
        gpio_bank_0,
        ModuleSlot::FrontRight,
        &mut registry,
        &mut bus_allocator,
    )
    .await;
    load_driver(
        module_1_id,
        gpio_bank_1,
        ModuleSlot::FrontLeft,
        &mut registry,
        &mut bus_allocator,
    )
    .await;
    load_driver(
        module_2_id,
        gpio_bank_2,
        ModuleSlot::BackRight,
        &mut registry,
        &mut bus_allocator,
    )
    .await;
    load_driver(
        module_3_id,
        gpio_bank_3,
        ModuleSlot::BackLeft,
        &mut registry,
        &mut bus_allocator,
    )
    .await;

    defmt::info!("drivers loaded, handing registry to core 0");
    REGISTRY_HANDOFF.signal(registry);
}
