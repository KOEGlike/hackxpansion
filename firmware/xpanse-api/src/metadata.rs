//! Metadata for apps about the drivers, like what physical slot the module is in, that the driver uses,
//! and what type of module is used by the driver.
//!
//! Module identification is resistor-coded: two detection resistors (`md0` and
//! `md1`) are read by the ADC at boot. The combination uniquely selects a
//! [`ModuleID`], which a [`crate::driver::Driver`] matches against its
//! [`DriverMeta::ID`](crate::driver::DriverMeta::ID) constant.

#[derive(Clone, Copy, Debug, PartialEq, Eq, defmt::Format)]
/// Physical slot that a module is plugged into.
pub enum ModuleSlot {
    FrontRight,
    FrontLeft,
    BackRight,
    BackLeft,
}

/// Resistor-coded module identifier: two 12-bit detection resistor values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, defmt::Format)]
pub struct ModuleID {
    /// Resistor on detection line 0.
    pub md0: ModuleDetectResistor,
    /// Resistor on detection line 1.
    pub md1: ModuleDetectResistor,
}

/// Standard E24-series resistor values used for module detection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, defmt::Format)]
pub enum ModuleDetectResistor {
    R1K,
    R1K1,
    R1K2,
    R1K3,
    R1K5,
    R1K6,
    R1K8,
    R2K,
    R2K2,
    R2K4,
    R2K7,
    R3K,
    R3K3,
    R3K6,
    R3K9,
    R4K3,
    R4K7,
    R5K1,
    R5K6,
    R6K2,
    R6K8,
    R7K5,
    R8K2,
    R9K1,
    R10K,
    R11K,
    R12K,
    R13K,
    R15K,
    R16K,
    R18K,
    R20K,
    R22K,
    R24K,
    R27K,
    R30K,
    R33K,
    R36K,
    R39K,
    R43K,
    R47K,
    R51K,
    R56K,
    R62K,
    R68K,
    R75K,
    R82K,
    R91K,
    R100K,
}

/// Fixed 10 kΩ pull-down resistor on the module-detection line.
pub const BOTTOM_RESISTOR: f64 = 10_000.0;

/// Supply voltage (nominal 3.3 V) used for the resistor-divider ADC reading.
pub const AVDD: f64 = 3.3;

impl ModuleDetectResistor {
    /// Convert a measured divider voltage back to the closest standard resistor value.
    ///
    /// Returns `None` if the voltage is out of range (≤ 0 V or ≥ `AVDD`) or the
    /// computed resistance falls outside the E24 range.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use xpanse_api::metadata::{ModuleDetectResistor, AVDD, BOTTOM_RESISTOR};
    ///
    /// // A 10 kΩ resistor with the 10 kΩ pull-down gives half of AVDD.
    /// let voltage = AVDD / 2.0;
    /// assert_eq!(ModuleDetectResistor::from_voltage(voltage), Some(ModuleDetectResistor::R10K));
    /// ```
    pub fn from_voltage(voltage: f64) -> Option<Self> {
        if !voltage.is_finite() || voltage <= 0.0 || voltage >= AVDD {
            return None;
        }

        let calculated_resistor = (AVDD * BOTTOM_RESISTOR) / voltage - BOTTOM_RESISTOR;
        if !calculated_resistor.is_finite() || !(950.0..=105_000.0).contains(&calculated_resistor) {
            return None;
        }

        let (_, closest_resistor) = RESISTOR_MAP
            .iter()
            .min_by(|(val_a, _), (val_b, _)| {
                let diff_a = (val_a - calculated_resistor).abs();
                let diff_b = (val_b - calculated_resistor).abs();

                diff_a
                    .partial_cmp(&diff_b)
                    .unwrap_or(core::cmp::Ordering::Equal)
            })
            .expect("RESISTOR_MAP should never be empty");

        Some(*closest_resistor)
    }
}

const RESISTOR_MAP: &[(f64, ModuleDetectResistor)] = &[
    (1_000.0, ModuleDetectResistor::R1K),
    (1_100.0, ModuleDetectResistor::R1K1),
    (1_200.0, ModuleDetectResistor::R1K2),
    (1_300.0, ModuleDetectResistor::R1K3),
    (1_500.0, ModuleDetectResistor::R1K5),
    (1_600.0, ModuleDetectResistor::R1K6),
    (1_800.0, ModuleDetectResistor::R1K8),
    (2_000.0, ModuleDetectResistor::R2K),
    (2_200.0, ModuleDetectResistor::R2K2),
    (2_400.0, ModuleDetectResistor::R2K4),
    (2_700.0, ModuleDetectResistor::R2K7),
    (3_000.0, ModuleDetectResistor::R3K),
    (3_300.0, ModuleDetectResistor::R3K3),
    (3_600.0, ModuleDetectResistor::R3K6),
    (3_900.0, ModuleDetectResistor::R3K9),
    (4_300.0, ModuleDetectResistor::R4K3),
    (4_700.0, ModuleDetectResistor::R4K7),
    (5_100.0, ModuleDetectResistor::R5K1),
    (5_600.0, ModuleDetectResistor::R5K6),
    (6_200.0, ModuleDetectResistor::R6K2),
    (6_800.0, ModuleDetectResistor::R6K8),
    (7_500.0, ModuleDetectResistor::R7K5),
    (8_200.0, ModuleDetectResistor::R8K2),
    (9_100.0, ModuleDetectResistor::R9K1),
    (10_000.0, ModuleDetectResistor::R10K),
    (11_000.0, ModuleDetectResistor::R11K),
    (12_000.0, ModuleDetectResistor::R12K),
    (13_000.0, ModuleDetectResistor::R13K),
    (15_000.0, ModuleDetectResistor::R15K),
    (16_000.0, ModuleDetectResistor::R16K),
    (18_000.0, ModuleDetectResistor::R18K),
    (20_000.0, ModuleDetectResistor::R20K),
    (22_000.0, ModuleDetectResistor::R22K),
    (24_000.0, ModuleDetectResistor::R24K),
    (27_000.0, ModuleDetectResistor::R27K),
    (30_000.0, ModuleDetectResistor::R30K),
    (33_000.0, ModuleDetectResistor::R33K),
    (36_000.0, ModuleDetectResistor::R36K),
    (39_000.0, ModuleDetectResistor::R39K),
    (43_000.0, ModuleDetectResistor::R4K3),
    (47_000.0, ModuleDetectResistor::R47K),
    (51_000.0, ModuleDetectResistor::R51K),
    (56_000.0, ModuleDetectResistor::R56K),
    (62_000.0, ModuleDetectResistor::R62K),
    (68_000.0, ModuleDetectResistor::R68K),
    (75_000.0, ModuleDetectResistor::R75K),
    (82_000.0, ModuleDetectResistor::R82K),
    (91_000.0, ModuleDetectResistor::R91K),
    (100_000.0, ModuleDetectResistor::R100K),
];

impl From<ModuleDetectResistor> for f64 {
    fn from(val: ModuleDetectResistor) -> Self {
        match val {
            ModuleDetectResistor::R1K => 1_000.0,
            ModuleDetectResistor::R1K1 => 1_100.0,
            ModuleDetectResistor::R1K2 => 1_200.0,
            ModuleDetectResistor::R1K3 => 1_300.0,
            ModuleDetectResistor::R1K5 => 1_500.0,
            ModuleDetectResistor::R1K6 => 1_600.0,
            ModuleDetectResistor::R1K8 => 1_800.0,
            ModuleDetectResistor::R2K => 2_000.0,
            ModuleDetectResistor::R2K2 => 2_200.0,
            ModuleDetectResistor::R2K4 => 2_400.0,
            ModuleDetectResistor::R2K7 => 2_700.0,
            ModuleDetectResistor::R3K => 3_000.0,
            ModuleDetectResistor::R3K3 => 3_300.0,
            ModuleDetectResistor::R3K6 => 3_600.0,
            ModuleDetectResistor::R3K9 => 3_900.0,
            ModuleDetectResistor::R4K3 => 4_300.0,
            ModuleDetectResistor::R4K7 => 4_700.0,
            ModuleDetectResistor::R5K1 => 5_100.0,
            ModuleDetectResistor::R5K6 => 5_600.0,
            ModuleDetectResistor::R6K2 => 6_200.0,
            ModuleDetectResistor::R6K8 => 6_800.0,
            ModuleDetectResistor::R7K5 => 7_500.0,
            ModuleDetectResistor::R8K2 => 8_200.0,
            ModuleDetectResistor::R9K1 => 9_100.0,
            ModuleDetectResistor::R10K => 10_000.0,
            ModuleDetectResistor::R11K => 11_000.0,
            ModuleDetectResistor::R12K => 12_000.0,
            ModuleDetectResistor::R13K => 13_000.0,
            ModuleDetectResistor::R15K => 15_000.0,
            ModuleDetectResistor::R16K => 16_000.0,
            ModuleDetectResistor::R18K => 18_000.0,
            ModuleDetectResistor::R20K => 20_000.0,
            ModuleDetectResistor::R22K => 22_000.0,
            ModuleDetectResistor::R24K => 24_000.0,
            ModuleDetectResistor::R27K => 27_000.0,
            ModuleDetectResistor::R30K => 30_000.0,
            ModuleDetectResistor::R33K => 33_000.0,
            ModuleDetectResistor::R36K => 36_000.0,
            ModuleDetectResistor::R39K => 39_000.0,
            ModuleDetectResistor::R43K => 43_000.0,
            ModuleDetectResistor::R47K => 47_000.0,
            ModuleDetectResistor::R51K => 51_000.0,
            ModuleDetectResistor::R56K => 56_000.0,
            ModuleDetectResistor::R62K => 62_000.0,
            ModuleDetectResistor::R68K => 68_000.0,
            ModuleDetectResistor::R75K => 75_000.0,
            ModuleDetectResistor::R82K => 82_000.0,
            ModuleDetectResistor::R91K => 91_000.0,
            ModuleDetectResistor::R100K => 100_000.0,
        }
    }
}
