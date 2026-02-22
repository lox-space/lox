// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Radio receiver models (simple and complex).

use lox_core::units::{Angle, Decibel, Frequency, Kelvin};

use crate::ROOM_TEMPERATURE;
use crate::antenna::AntennaGain;

/// A simple receiver with a known system noise temperature.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SimpleReceiver {
    /// Receive frequency.
    pub frequency: Frequency,
    /// System noise temperature in Kelvin.
    pub system_noise_temperature: Kelvin,
}

/// A complex receiver with detailed noise and gain parameters.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComplexReceiver {
    /// Receive frequency.
    pub frequency: Frequency,
    /// Antenna noise temperature in Kelvin.
    pub antenna_noise_temperature: Kelvin,
    /// LNA gain.
    pub lna_gain: Decibel,
    /// LNA noise figure.
    pub lna_noise_figure: Decibel,
    /// Receiver noise figure.
    pub noise_figure: Decibel,
    /// Receiver chain loss.
    pub loss: Decibel,
    /// Demodulator implementation loss.
    pub demodulator_loss: Decibel,
    /// Other implementation losses.
    pub implementation_loss: Decibel,
}

impl ComplexReceiver {
    /// Returns the receiver noise temperature in Kelvin.
    ///
    /// T_rx = T_room · (10^(NF/10) − 1)
    pub fn noise_temperature(&self) -> Kelvin {
        ROOM_TEMPERATURE * (self.noise_figure.to_linear() - 1.0)
    }

    /// Returns the system noise temperature in Kelvin.
    ///
    /// L = 10^(−loss/10) (linear loss factor)
    /// T_sys = T_ant · L + T_room · (1 − L) + T_rx
    pub fn system_noise_temperature(&self) -> Kelvin {
        let loss_linear = (-self.loss).to_linear(); // 10^(-loss_dB/10)
        let t_rx = self.noise_temperature();
        self.antenna_noise_temperature * loss_linear + ROOM_TEMPERATURE * (1.0 - loss_linear) + t_rx
    }
}

/// A receiver, either simple (known T_sys) or complex (detailed parameters).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum Receiver {
    Simple(SimpleReceiver),
    Complex(ComplexReceiver),
}

impl Receiver {
    /// Returns the system noise temperature in Kelvin.
    pub fn system_noise_temperature(&self) -> Kelvin {
        match self {
            Receiver::Simple(r) => r.system_noise_temperature,
            Receiver::Complex(r) => r.system_noise_temperature(),
        }
    }

    /// Returns the total receiver gain in dB.
    ///
    /// For a simple receiver, this is just the antenna gain.
    /// For a complex receiver: G_ant + G_lna − loss − demod_loss − impl_loss.
    pub fn total_gain(&self, antenna: &impl AntennaGain, angle: Angle) -> Decibel {
        match self {
            Receiver::Simple(r) => antenna.gain(r.frequency, angle),
            Receiver::Complex(r) => {
                antenna.gain(r.frequency, angle) + r.lna_gain
                    - r.loss
                    - r.demodulator_loss
                    - r.implementation_loss
            }
        }
    }

    /// Returns the gain-to-noise-temperature ratio (G/T) in dB/K.
    ///
    /// G/T = G_total − 10·log₁₀(T_sys) [− NF_lna for complex receivers]
    pub fn gain_to_noise_temperature(&self, antenna: &impl AntennaGain, angle: Angle) -> Decibel {
        let g_total = self.total_gain(antenna, angle);
        let t_sys = self.system_noise_temperature();
        match self {
            Receiver::Simple(_) => g_total - Decibel::from_linear(t_sys),
            Receiver::Complex(r) => g_total - Decibel::from_linear(t_sys) - r.lna_noise_figure,
        }
    }

    /// Returns the receive frequency.
    pub fn frequency(&self) -> Frequency {
        match self {
            Receiver::Simple(r) => r.frequency,
            Receiver::Complex(r) => r.frequency,
        }
    }
}

#[cfg(test)]
mod tests {
    use lox_core::units::{DecibelUnits, FrequencyUnits};
    use lox_test_utils::assert_approx_eq;

    use crate::antenna::SimpleAntenna;

    use super::*;

    #[test]
    fn test_complex_receiver_noise_temperature() {
        // NF = 5 dB → T_rx = 290 * (10^(5/10) - 1) = 290 * (3.16228 - 1) = 290 * 2.16228 = 627.06
        let rx = ComplexReceiver {
            frequency: 29.0.ghz(),
            antenna_noise_temperature: 265.0,
            lna_gain: 30.0.db(),
            lna_noise_figure: 1.0.db(),
            noise_figure: 5.0.db(),
            loss: 3.0.db(),
            demodulator_loss: 0.0.db(),
            implementation_loss: 0.0.db(),
        };
        assert_approx_eq!(rx.noise_temperature(), 627.0605214, rtol <= 1e-6);
    }

    #[test]
    fn test_complex_receiver_system_noise_temperature() {
        // NF=5dB, loss=3dB, T_ant=265K
        // L = 10^(-3/10) = 0.50119
        // T_rx = 290 * (10^(5/10) - 1) = 627.0605214
        // T_sys = 265 * 0.50119 + 290 * (1 - 0.50119) + 627.0605214
        //       = 132.815 + 144.655 + 627.061 = 904.531
        let rx = ComplexReceiver {
            frequency: 29.0.ghz(),
            antenna_noise_temperature: 265.0,
            lna_gain: 30.0.db(),
            lna_noise_figure: 1.0.db(),
            noise_figure: 5.0.db(),
            loss: 3.0.db(),
            demodulator_loss: 0.0.db(),
            implementation_loss: 0.0.db(),
        };
        assert_approx_eq!(rx.system_noise_temperature(), 904.53084061, rtol <= 1e-6);
    }

    #[test]
    fn test_simple_receiver_gt() {
        // SimpleReceiver with T_sys=500K, antenna gain=30dBi
        // G/T = 30 - 10*log10(500) = 30 - 26.9897 = 3.0103 dB/K
        let antenna = SimpleAntenna {
            gain: 30.0.db(),
            beamwidth: Angle::degrees(1.0),
        };
        let rx = Receiver::Simple(SimpleReceiver {
            frequency: 29.0.ghz(),
            system_noise_temperature: 500.0,
        });
        let gt = rx.gain_to_noise_temperature(&antenna, Angle::radians(0.0));
        assert_approx_eq!(gt.as_f64(), 3.0103, atol <= 0.001);
    }

    #[test]
    fn test_complex_receiver_total_gain() {
        // Antenna gain=30dBi, LNA=20dB, loss=3dB, demod=1dB, impl=0.5dB
        // G_total = 30 + 20 - 3 - 1 - 0.5 = 45.5 dB
        let antenna = SimpleAntenna {
            gain: 30.0.db(),
            beamwidth: Angle::degrees(1.0),
        };
        let rx = Receiver::Complex(ComplexReceiver {
            frequency: 29.0.ghz(),
            antenna_noise_temperature: 265.0,
            lna_gain: 20.0.db(),
            lna_noise_figure: 1.0.db(),
            noise_figure: 5.0.db(),
            loss: 3.0.db(),
            demodulator_loss: 1.0.db(),
            implementation_loss: 0.5.db(),
        });
        let g_total = rx.total_gain(&antenna, Angle::radians(0.0));
        assert_approx_eq!(g_total.as_f64(), 45.5, atol <= 1e-10);
    }
}
