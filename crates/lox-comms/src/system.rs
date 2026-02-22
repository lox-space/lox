// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Communication system composing antenna, transmitter, and receiver.

use lox_core::units::{Angle, Decibel, Distance, Frequency};

use crate::BOLTZMANN_CONSTANT;
use crate::antenna::Antenna;
use crate::receiver::Receiver;
use crate::transmitter::Transmitter;
use crate::utils::free_space_path_loss;

/// A communication system combining an antenna with optional transmitter and receiver.
pub struct CommunicationSystem {
    /// The antenna.
    pub antenna: Antenna,
    /// The receiver (if this system can receive).
    pub receiver: Option<Receiver>,
    /// The transmitter (if this system can transmit).
    pub transmitter: Option<Transmitter>,
}

impl CommunicationSystem {
    /// Computes the carrier-to-noise density ratio (C/N₀) in dB·Hz.
    ///
    /// C/N₀ = EIRP + G/T − FSPL − losses − 10·log₁₀(k_B)
    ///
    /// `self` is the transmitting system, `rx` is the receiving system.
    pub fn carrier_to_noise_density(
        &self,
        rx: &CommunicationSystem,
        losses: Decibel,
        range: Distance,
        tx_angle: Angle,
        rx_angle: Angle,
    ) -> Decibel {
        let tx = self
            .transmitter
            .as_ref()
            .expect("transmitting system must have a transmitter");
        let receiver = rx
            .receiver
            .as_ref()
            .expect("receiving system must have a receiver");

        let eirp = tx.eirp(&self.antenna, tx_angle);
        let gt = receiver.gain_to_noise_temperature(&rx.antenna, rx_angle);
        let fspl = free_space_path_loss(range, tx.frequency);
        let k_db = Decibel::from_linear(BOLTZMANN_CONSTANT);

        eirp + gt - fspl - losses - k_db
    }

    /// Computes the received carrier power in dBW.
    ///
    /// P_rx = EIRP − FSPL − losses + G_rx_total
    pub fn carrier_power(
        &self,
        rx: &CommunicationSystem,
        losses: Decibel,
        range: Distance,
        tx_angle: Angle,
        rx_angle: Angle,
    ) -> Decibel {
        let tx = self
            .transmitter
            .as_ref()
            .expect("transmitting system must have a transmitter");
        let receiver = rx
            .receiver
            .as_ref()
            .expect("receiving system must have a receiver");

        let eirp = tx.eirp(&self.antenna, tx_angle);
        let fspl = free_space_path_loss(range, tx.frequency);
        let g_rx = receiver.total_gain(&rx.antenna, rx_angle);

        eirp - fspl - losses + g_rx
    }

    /// Computes the noise power in dBW for a given bandwidth.
    ///
    /// P_noise = 10·log₁₀(T_sys · k_B · BW)
    pub fn noise_power(&self, bandwidth_hz: f64) -> Decibel {
        let receiver = self.receiver.as_ref().expect("system must have a receiver");

        let t_sys = receiver.system_noise_temperature();
        let p_noise_w = t_sys * BOLTZMANN_CONSTANT * bandwidth_hz;
        Decibel::from_linear(p_noise_w)
    }

    /// Returns the transmit frequency, if this system has a transmitter.
    pub fn tx_frequency(&self) -> Option<Frequency> {
        self.transmitter.as_ref().map(|tx| tx.frequency)
    }

    /// Returns the receive frequency, if this system has a receiver.
    pub fn rx_frequency(&self) -> Option<Frequency> {
        self.receiver.as_ref().map(|rx| rx.frequency())
    }
}

#[cfg(test)]
mod tests {
    use lox_core::units::{DecibelUnits, FrequencyUnits};
    use lox_test_utils::assert_approx_eq;

    use crate::antenna::SimpleAntenna;
    use crate::receiver::SimpleReceiver;

    use super::*;

    fn tx_system() -> CommunicationSystem {
        CommunicationSystem {
            antenna: Antenna::Simple(SimpleAntenna {
                gain: 46.0.db(),
                beamwidth: Angle::degrees(0.7),
            }),
            receiver: None,
            transmitter: Some(Transmitter::new(29.0.ghz(), 10.0, 1.0.db(), 0.0.db())),
        }
    }

    fn rx_system() -> CommunicationSystem {
        CommunicationSystem {
            antenna: Antenna::Simple(SimpleAntenna {
                gain: 30.0.db(),
                beamwidth: Angle::degrees(3.0),
            }),
            receiver: Some(Receiver::Simple(SimpleReceiver {
                frequency: 29.0.ghz(),
                system_noise_temperature: 500.0,
            })),
            transmitter: None,
        }
    }

    #[test]
    fn test_carrier_to_noise_density() {
        // TX: 46 dBi antenna, 10 W (10 dBW), 1 dB line loss → EIRP = 46 + 10 - 1 = 55 dBW
        // RX: 30 dBi antenna, T_sys=500K → G/T = 30 - 26.99 = 3.01 dB/K
        // FSPL at 1000 km, 29 GHz ≈ 181.696 dB
        // k_B = -228.599 dBW/K/Hz
        // C/N0 = 55 + 3.01 - 181.696 - 0 - (-228.599) = 104.913 dB·Hz
        let tx = tx_system();
        let rx = rx_system();
        let c_n0 = tx.carrier_to_noise_density(
            &rx,
            0.0.db(),
            Distance::kilometers(1000.0),
            Angle::radians(0.0),
            Angle::radians(0.0),
        );
        // Verify within reasonable tolerance (some rounding in intermediate values)
        assert_approx_eq!(c_n0.as_f64(), 104.913, atol <= 0.1);
    }

    #[test]
    fn test_carrier_power() {
        // P_rx = EIRP - FSPL - losses + G_rx
        // = 55 - 181.696 - 0 + 30 = -96.696 dBW
        let tx = tx_system();
        let rx = rx_system();
        let p_rx = tx.carrier_power(
            &rx,
            0.0.db(),
            Distance::kilometers(1000.0),
            Angle::radians(0.0),
            Angle::radians(0.0),
        );
        assert_approx_eq!(p_rx.as_f64(), -96.696, atol <= 0.1);
    }

    #[test]
    fn test_noise_power() {
        // P_noise = 10*log10(500 * 1.38064852e-23 * 1e6)
        //         = 10*log10(6.90324e-15)
        //         = -141.61 dBW
        let rx = rx_system();
        let p_noise = rx.noise_power(1e6);
        assert_approx_eq!(p_noise.as_f64(), -141.61, atol <= 0.01);
    }

    #[test]
    fn test_c_n0_consistency() {
        // C/N0 should equal P_rx - P_noise + 10*log10(BW)
        let tx = tx_system();
        let rx = rx_system();
        let range = Distance::kilometers(1000.0);
        let bw = 1e6;

        let c_n0 = tx.carrier_to_noise_density(
            &rx,
            0.0.db(),
            range,
            Angle::radians(0.0),
            Angle::radians(0.0),
        );
        let p_rx = tx.carrier_power(
            &rx,
            0.0.db(),
            range,
            Angle::radians(0.0),
            Angle::radians(0.0),
        );
        let p_noise = rx.noise_power(bw);

        // C/N0 = P_rx - P_noise + 10*log10(BW) (converting from C/N to C/N0)
        let c_n0_from_power = p_rx - p_noise + Decibel::from_linear(bw);
        assert_approx_eq!(c_n0.as_f64(), c_n0_from_power.as_f64(), atol <= 0.01);
    }
}
