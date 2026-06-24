//! Global ADC service.
//!
//! Provides a global singleton backed by the RP235x ADC block. Initialised once
//! at startup via [`init_adc`], then any driver or app can:
//!
//! * read the chip temperature via [`read_temperature`] / [`read_temperature_raw`]
//! * sample an ADC-capable GPIO pin via [`read_adc_pin`] / [`read_adc_voltage`]
//!
//! The ADC is shared through an async [`Mutex`](embassy_sync::mutex::Mutex), so
//! concurrent reads from multiple tasks are safe — they simply queue.
//!
//! [`read_adc_pin`] borrows the pin via `Peri::reborrow()`, so the driver
//! retains ownership of its `Peri<'static, …>` — the pin's pad is temporarily
//! reconfigured for ADC during the read and restored when the temporary
//! [`Channel`] is dropped.

use core::ptr::null_mut;
use core::sync::atomic::{AtomicPtr, Ordering};

use embassy_rp::adc::{self, Adc, AdcPin, Async, Channel};
use embassy_rp::gpio::Pull;
use embassy_rp::peripherals::{ADC, ADC_TEMP_SENSOR};
use embassy_rp::Peri;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use static_cell::StaticCell;

use crate::metadata::AVDD;

embassy_rp::bind_interrupts!(struct AdcIrqs {
    ADC_IRQ_FIFO => embassy_rp::adc::InterruptHandler;
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum AdcError {
    /// [`init_adc`] has not been called yet.
    NotInitialized,
    /// ADC conversion failed.
    ConversionFailed,
}

/// 12-bit ADC max value.
const ADC_MAX: f64 = 4095.0;

/// Temperature sensor calibration constants (RP235x datasheet §12.10.4).
const T_REF: f64 = 27.0;
const V_REF: f64 = 0.706;
const SLOPE: f64 = 0.001721;

struct AdcService {
    adc: Adc<'static, Async>,
    /// Permanent channel for the internal temperature sensor.
    temp_channel: Channel<'static>,
}

impl AdcService {
    fn new(adc_peri: Peri<'static, ADC>, temp_peri: Peri<'static, ADC_TEMP_SENSOR>) -> Self {
        let adc = Adc::new(adc_peri, AdcIrqs, adc::Config::default());
        let temp_channel = Channel::new_temp_sensor(temp_peri);
        Self { adc, temp_channel }
    }

    async fn read_temp_raw(&mut self) -> Result<u16, AdcError> {
        self.adc
            .read(&mut self.temp_channel)
            .await
            .map_err(|_| AdcError::ConversionFailed)
    }

    async fn read_temp_celsius(&mut self) -> Result<f64, AdcError> {
        let raw = self.read_temp_raw().await?;
        let voltage = raw as f64 * AVDD / ADC_MAX;
        let temp = T_REF - (voltage - V_REF) / SLOPE;
        Ok(temp)
    }

    async fn read_pin_raw(&mut self, pin: Peri<'_, impl AdcPin>, pull: Pull) -> Result<u16, AdcError> {
        let mut channel = Channel::new_pin(pin, pull);
        self.adc
            .read(&mut channel)
            .await
            .map_err(|_| AdcError::ConversionFailed)
    }
}

type AdcMutex = Mutex<CriticalSectionRawMutex, AdcService>;

static ADC_CELL: StaticCell<AdcMutex> = StaticCell::new();
static ADC_PTR: AtomicPtr<AdcMutex> = AtomicPtr::new(null_mut());

/// Initialise the global ADC service. Must be called exactly once before any
/// [`read_temperature`], [`read_temperature_raw`], [`read_adc_pin`] or
/// [`read_adc_voltage`] call.
pub fn init_adc(adc: Peri<'static, ADC>, temp_sensor: Peri<'static, ADC_TEMP_SENSOR>) {
    let mutex = ADC_CELL.init(Mutex::new(AdcService::new(adc, temp_sensor)));
    ADC_PTR.store(mutex as *mut AdcMutex, Ordering::Release);
}

/// Read the chip temperature in degrees Celsius.
pub async fn read_temperature() -> Result<f64, AdcError> {
    let mutex = adc_mutex().ok_or(AdcError::NotInitialized)?;
    let mut guard = mutex.lock().await;
    guard.read_temp_celsius().await
}

/// Read the raw 12-bit ADC value from the internal temperature sensor.
pub async fn read_temperature_raw() -> Result<u16, AdcError> {
    let mutex = adc_mutex().ok_or(AdcError::NotInitialized)?;
    let mut guard = mutex.lock().await;
    guard.read_temp_raw().await
}

/// Read the raw 12-bit ADC value from a GPIO pin. The pin is borrowed (via
/// `reborrow`), so the caller retains ownership of its `Peri<'static, …>`.
///
/// The `pull` argument configures the pin's pull resistor during the read; the
/// pad is restored to its GPIO defaults when the internal channel is dropped.
pub async fn read_adc_pin(pin: Peri<'_, impl AdcPin>, pull: Pull) -> Result<u16, AdcError> {
    let mutex = adc_mutex().ok_or(AdcError::NotInitialized)?;
    let mut guard = mutex.lock().await;
    guard.read_pin_raw(pin, pull).await
}

/// Read an ADC GPIO pin and convert the result to a voltage (0..=AVDD volts).
pub async fn read_adc_voltage(pin: Peri<'_, impl AdcPin>, pull: Pull) -> Result<f64, AdcError> {
    let raw = read_adc_pin(pin, pull).await?;
    Ok(raw as f64 * AVDD / ADC_MAX)
}

fn adc_mutex() -> Option<&'static AdcMutex> {
    let ptr = ADC_PTR.load(Ordering::Acquire);
    if ptr.is_null() {
        return None;
    }
    // SAFETY: `ptr` was stored by `init_adc` from a `&'static AdcMutex`
    // returned by `StaticCell::init`. The `StaticCell` lives for `'static` and
    // is never moved. The `Acquire` load above synchronises with the `Release`
    // store in `init_adc`, so the pointed-to `AdcMutex` is fully initialised
    // and visible.
    Some(unsafe { &*ptr })
}
