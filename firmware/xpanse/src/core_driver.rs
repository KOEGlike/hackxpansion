use xpanse_api::gpio_bank::GpioBank;
use xpanse_api::registry::Registry;

use crate::load_driver::load_driver;
use crate::{adc::init_adc, adc_mapping, resource_split::*};
use xpanse_api::bus::allocator::BusAllocator;
use xpanse_api::interfaces::adc;
use xpanse_api::metadata::{ModuleID, ModuleSlot};

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::Timer;

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

    let module_ids = {
        let module_adc = match init_adc(
            remaining_peris.i2c1.reborrow(),
            i2c_pins.sda.reborrow(),
            i2c_pins.scl.reborrow(),
        )
        .await
        {
            Ok(adc) => Some(adc),
            Err(error) => {
                defmt::warn!("module ADC initialization attempt 1 failed: {:?}", error);
                Timer::after_millis(20).await;
                match init_adc(
                    remaining_peris.i2c1.reborrow(),
                    i2c_pins.sda.reborrow(),
                    i2c_pins.scl.reborrow(),
                )
                .await
                {
                    Ok(adc) => Some(adc),
                    Err(error) => {
                        defmt::warn!("module ADC initialization attempt 2 failed: {:?}", error);
                        Timer::after_millis(20).await;
                        match init_adc(
                            remaining_peris.i2c1.reborrow(),
                            i2c_pins.sda.reborrow(),
                            i2c_pins.scl.reborrow(),
                        )
                        .await
                        {
                            Ok(adc) => Some(adc),
                            Err(error) => {
                                defmt::warn!(
                                    "module ADC initialization attempt 3 failed: {:?}",
                                    error
                                );
                                None
                            }
                        }
                    }
                }
            }
        };

        match module_adc {
            Some(mut module_adc) => [
                detect_module(&mut module_adc, ModuleSlot::FrontRight).await,
                detect_module(&mut module_adc, ModuleSlot::FrontLeft).await,
                detect_module(&mut module_adc, ModuleSlot::BackRight).await,
                detect_module(&mut module_adc, ModuleSlot::BackLeft).await,
            ],
            None => {
                defmt::error!("module ADC initialization failed after three attempts");
                [None; 4]
            }
        }
    };

    adc::init_adc(remaining_peris.adc, remaining_peris.adc_temp);

    let mut bus_allocator = BusAllocator::new(
        None,
        Some(remaining_peris.spi1),
        Some(remaining_peris.i2c0),
        Some(remaining_peris.i2c1),
        Some(remaining_peris.uart0),
        Some(remaining_peris.uart1),
        xpanse_api::bus::allocator::DmaPool {
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
        module_ids[0],
        gpio_bank_0,
        ModuleSlot::FrontRight,
        &mut registry,
        &mut bus_allocator,
    )
    .await;
    load_driver(
        module_ids[1],
        gpio_bank_1,
        ModuleSlot::FrontLeft,
        &mut registry,
        &mut bus_allocator,
    )
    .await;
    load_driver(
        module_ids[2],
        gpio_bank_2,
        ModuleSlot::BackRight,
        &mut registry,
        &mut bus_allocator,
    )
    .await;
    load_driver(
        module_ids[3],
        gpio_bank_3,
        ModuleSlot::BackLeft,
        &mut registry,
        &mut bus_allocator,
    )
    .await;

    defmt::info!("drivers loaded, handing registry to core 0");
    REGISTRY_HANDOFF.signal(registry);
}

async fn detect_module(adc: &mut crate::adc::Adc<'_>, slot: ModuleSlot) -> Option<ModuleID> {
    for attempt in 1..=3 {
        match adc_mapping::map_adc(adc, slot).await {
            Ok(id) => return id,
            Err(error) => {
                defmt::warn!(
                    "module detection attempt {} failed for {:?}: {:?}",
                    attempt,
                    slot,
                    error
                );
                Timer::after_millis(10).await;
            }
        }
    }
    defmt::error!("module detection failed for {:?}", slot);
    None
}
