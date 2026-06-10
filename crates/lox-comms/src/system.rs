// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Communication system composing antenna, transmitter, and receiver.

use lox_core::units::{Angle, Decibel, Distance, Frequency};

use crate::BOLTZMANN_CONSTANT;
use crate::LinkBudgetError;
use crate::antenna::Antenna;
use crate::receiver::Receiver;
use crate::transmitter::Transmitter;
use crate::utils::free_space_path_loss;

/// A communication system combining an optional antenna with optional transmitter and receiver.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommunicationSystem {
    /// The antenna (required for component-tier TX/RX; must be absent for lumped `Eirp`/`Gt`).
    pub antenna: Option<Antenna>,
    /// The receiver (if this system can receive).
    pub receiver: Option<Receiver>,
    /// The transmitter (if this system can transmit).
    pub transmitter: Option<Transmitter>,
}

impl CommunicationSystem {
    /// Creates a system whose transmitter is described by an aggregate EIRP figure.
    pub fn eirp_only(tx: crate::transmitter::EirpTransmitter) -> Self {
        Self {
            antenna: None,
            receiver: None,
            transmitter: Some(Transmitter::Eirp(tx)),
        }
    }

    /// Creates a system whose receiver is described by an aggregate G/T figure.
    pub fn gt_only(rx: crate::receiver::GtReceiver) -> Self {
        Self {
            antenna: None,
            receiver: Some(Receiver::Gt(rx)),
            transmitter: None,
        }
    }

    /// Creates a system from an antenna plus a component-tier amplifier transmitter.
    pub fn amplifier_with(antenna: Antenna, tx: crate::transmitter::AmplifierTransmitter) -> Self {
        Self {
            antenna: Some(antenna),
            receiver: None,
            transmitter: Some(Transmitter::Amplifier(tx)),
        }
    }

    /// Creates a system from an antenna plus a component-tier receiver
    /// (either `NoiseTemperature` or `Cascade`).
    ///
    /// # Panics (debug builds only)
    ///
    /// Panics if `rx` is [`Receiver::Gt`]; use [`Self::gt_only`] for lumped G/T receivers.
    pub fn receiver_with(antenna: Antenna, rx: Receiver) -> Self {
        debug_assert!(
            !matches!(rx, Receiver::Gt(_)),
            "receiver_with: use gt_only for lumped G/T receivers"
        );
        Self {
            antenna: Some(antenna),
            receiver: Some(rx),
            transmitter: None,
        }
    }

    /// Computes the carrier-to-noise density ratio (C/N₀) in dB·Hz.
    ///
    /// C/N₀ = EIRP + G/T − FSPL − losses − 10·log₁₀(k_B)
    ///
    /// Returns an error if `self` has no transmitter or `rx` has no receiver.
    pub fn carrier_to_noise_density(
        &self,
        rx: &CommunicationSystem,
        losses: Decibel,
        range: Distance,
        tx_angle: Angle,
        rx_angle: Angle,
    ) -> Result<Decibel, LinkBudgetError> {
        let tx = self
            .transmitter
            .as_ref()
            .ok_or(LinkBudgetError::MissingTransmitter)?;
        let receiver = rx
            .receiver
            .as_ref()
            .ok_or(LinkBudgetError::MissingReceiver)?;

        let tx_freq = tx.frequency();
        let rx_freq = receiver.frequency();
        if tx_freq != rx_freq {
            return Err(LinkBudgetError::FrequencyMismatch {
                tx: tx_freq,
                rx: rx_freq,
            });
        }

        let phi = Angle::ZERO;
        let eirp = tx_eirp(tx, &self.antenna, tx_angle, phi)?;
        let gt = rx_gt(receiver, &rx.antenna, rx_angle, phi)?;
        let fspl = free_space_path_loss(range, tx.frequency());
        let k_db = Decibel::from_linear(BOLTZMANN_CONSTANT);

        Ok(eirp + gt - fspl - losses - k_db)
    }

    /// Computes the received carrier power in dBW.
    ///
    /// P_rx = EIRP − FSPL − losses + G_rx_total
    ///
    /// Returns `Ok(None)` for lumped `Gt` receivers — the absolute receive gain
    /// is not recoverable from a G/T figure.
    ///
    /// Returns an error if `self` has no transmitter or `rx` has no receiver.
    pub fn carrier_power(
        &self,
        rx: &CommunicationSystem,
        losses: Decibel,
        range: Distance,
        tx_angle: Angle,
        rx_angle: Angle,
    ) -> Result<Option<Decibel>, LinkBudgetError> {
        let tx = self
            .transmitter
            .as_ref()
            .ok_or(LinkBudgetError::MissingTransmitter)?;
        let receiver = rx
            .receiver
            .as_ref()
            .ok_or(LinkBudgetError::MissingReceiver)?;

        let tx_freq = tx.frequency();
        let rx_freq = receiver.frequency();
        if tx_freq != rx_freq {
            return Err(LinkBudgetError::FrequencyMismatch {
                tx: tx_freq,
                rx: rx_freq,
            });
        }

        if matches!(receiver, Receiver::Gt(_)) {
            return Ok(None);
        }

        let antenna = rx.antenna.as_ref().ok_or(LinkBudgetError::MissingAntenna)?;
        let phi = Angle::ZERO;
        let eirp = tx_eirp(tx, &self.antenna, tx_angle, phi)?;
        let fspl = free_space_path_loss(range, tx.frequency());
        let g_rx = receiver.total_gain(antenna, rx_angle, phi);
        Ok(Some(eirp - fspl - losses + g_rx))
    }

    /// Computes the noise power in dBW for a given bandwidth.
    ///
    /// P_noise = 10·log₁₀(T_sys · k_B · BW)
    ///
    /// Returns `Ok(None)` for lumped `Gt` receivers — the system noise temperature
    /// is not exposed separately by a G/T figure.
    ///
    /// Returns an error if `self` has no receiver.
    pub fn noise_power(&self, bandwidth_hz: f64) -> Result<Option<Decibel>, LinkBudgetError> {
        let receiver = self
            .receiver
            .as_ref()
            .ok_or(LinkBudgetError::MissingReceiver)?;

        if matches!(receiver, Receiver::Gt(_)) {
            return Ok(None);
        }

        let t_sys = receiver.system_noise_temperature();
        let p_noise_w = t_sys * BOLTZMANN_CONSTANT * bandwidth_hz;
        Ok(Some(Decibel::from_linear(p_noise_w)))
    }

    /// Returns the EIRP at the given off-boresight angle.
    ///
    /// For lumped `Eirp` transmitters, returns the stored figure; the angle is ignored.
    pub fn eirp_at(&self, angle: Angle) -> Result<Decibel, LinkBudgetError> {
        let tx = self
            .transmitter
            .as_ref()
            .ok_or(LinkBudgetError::MissingTransmitter)?;
        tx_eirp(tx, &self.antenna, angle, Angle::ZERO)
    }

    /// Returns the G/T at the given off-boresight angle.
    ///
    /// For lumped `Gt` receivers, returns the stored figure; the angle is ignored.
    pub fn gt_at(&self, angle: Angle) -> Result<Decibel, LinkBudgetError> {
        let rx = self
            .receiver
            .as_ref()
            .ok_or(LinkBudgetError::MissingReceiver)?;
        rx_gt(rx, &self.antenna, angle, Angle::ZERO)
    }

    /// Returns the transmit frequency, if this system has a transmitter.
    pub fn tx_frequency(&self) -> Option<Frequency> {
        self.transmitter.as_ref().map(|tx| tx.frequency())
    }

    /// Returns the receive frequency, if this system has a receiver.
    pub fn rx_frequency(&self) -> Option<Frequency> {
        self.receiver.as_ref().map(|rx| rx.frequency())
    }
}

fn tx_eirp(
    tx: &Transmitter,
    antenna: &Option<Antenna>,
    theta: Angle,
    phi: Angle,
) -> Result<Decibel, LinkBudgetError> {
    match tx {
        Transmitter::Eirp(t) => {
            if antenna.is_some() {
                return Err(LinkBudgetError::UnexpectedAntenna);
            }
            Ok(t.eirp)
        }
        Transmitter::Amplifier(t) => {
            let ant = antenna.as_ref().ok_or(LinkBudgetError::MissingAntenna)?;
            Ok(t.eirp(ant, theta, phi))
        }
    }
}

fn rx_gt(
    rx: &Receiver,
    antenna: &Option<Antenna>,
    theta: Angle,
    phi: Angle,
) -> Result<Decibel, LinkBudgetError> {
    match rx {
        Receiver::Gt(r) => {
            if antenna.is_some() {
                return Err(LinkBudgetError::UnexpectedAntenna);
            }
            Ok(r.gt)
        }
        Receiver::NoiseTemperature(_) | Receiver::Cascade(_) => {
            let ant = antenna.as_ref().ok_or(LinkBudgetError::MissingAntenna)?;
            Ok(rx.gain_to_noise_temperature(ant, theta, phi))
        }
    }
}

#[cfg(test)]
mod tests {
    use lox_core::units::{DecibelUnits, FrequencyUnits};
    use lox_test_utils::assert_approx_eq;

    use crate::antenna::ConstantAntenna;
    use crate::receiver::NoiseTempReceiver;
    use crate::transmitter::AmplifierTransmitter;

    use super::*;

    fn tx_system() -> CommunicationSystem {
        CommunicationSystem {
            antenna: Some(Antenna::Constant(ConstantAntenna { gain: 46.0.db() })),
            receiver: None,
            transmitter: Some(Transmitter::Amplifier(AmplifierTransmitter::new(
                29.0.ghz(),
                10.0,
                1.0.db(),
                0.0.db(),
            ))),
        }
    }

    fn rx_system() -> CommunicationSystem {
        CommunicationSystem {
            antenna: Some(Antenna::Constant(ConstantAntenna { gain: 30.0.db() })),
            receiver: Some(Receiver::NoiseTemperature(NoiseTempReceiver {
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
        let c_n0 = tx
            .carrier_to_noise_density(
                &rx,
                0.0.db(),
                Distance::kilometers(1000.0),
                Angle::ZERO,
                Angle::ZERO,
            )
            .unwrap();
        // Verify within reasonable tolerance (some rounding in intermediate values)
        assert_approx_eq!(c_n0.as_f64(), 104.913, atol <= 0.1);
    }

    #[test]
    fn test_carrier_power() {
        // P_rx = EIRP - FSPL - losses + G_rx
        // = 55 - 181.696 - 0 + 30 = -96.696 dBW
        let tx = tx_system();
        let rx = rx_system();
        let p_rx = tx
            .carrier_power(
                &rx,
                0.0.db(),
                Distance::kilometers(1000.0),
                Angle::ZERO,
                Angle::ZERO,
            )
            .unwrap()
            .expect("component receiver produces carrier_power");
        assert_approx_eq!(p_rx.as_f64(), -96.696, atol <= 0.1);
    }

    #[test]
    fn test_noise_power() {
        // P_noise = 10*log10(500 * BOLTZMANN_CONSTANT * 1e6)
        //         = 10*log10(500 * 1.380649e-23 * 1e6)
        //         = -141.61 dBW
        let rx = rx_system();
        let p_noise = rx
            .noise_power(1e6)
            .unwrap()
            .expect("component receiver produces noise_power");
        assert_approx_eq!(p_noise.as_f64(), -141.61, atol <= 0.01);
    }

    #[test]
    fn test_c_n0_consistency() {
        // C/N0 should equal P_rx - P_noise + 10*log10(BW)
        let tx = tx_system();
        let rx = rx_system();
        let range = Distance::kilometers(1000.0);
        let bw = 1e6;

        let c_n0 = tx
            .carrier_to_noise_density(&rx, 0.0.db(), range, Angle::ZERO, Angle::ZERO)
            .unwrap();
        let p_rx = tx
            .carrier_power(&rx, 0.0.db(), range, Angle::ZERO, Angle::ZERO)
            .unwrap()
            .expect("component receiver produces carrier_power");
        let p_noise = rx
            .noise_power(bw)
            .unwrap()
            .expect("component receiver produces noise_power");

        // C/N0 = P_rx - P_noise + 10*log10(BW) (converting from C/N to C/N0)
        let c_n0_from_power = p_rx - p_noise + Decibel::from_linear(bw);
        assert_approx_eq!(c_n0.as_f64(), c_n0_from_power.as_f64(), atol <= 0.01);
    }

    #[test]
    fn test_eirp_gt_c_n0_lumped() {
        use crate::receiver::GtReceiver;
        use crate::transmitter::EirpTransmitter;

        let tx = CommunicationSystem::eirp_only(EirpTransmitter {
            frequency: 29.0.ghz(),
            eirp: 55.0.db(),
        });
        let rx = CommunicationSystem::gt_only(GtReceiver {
            frequency: 29.0.ghz(),
            gt: 3.01.db(),
        });
        let c_n0 = tx
            .carrier_to_noise_density(
                &rx,
                0.0.db(),
                Distance::kilometers(1000.0),
                Angle::ZERO,
                Angle::ZERO,
            )
            .unwrap();
        // EIRP=55, G/T=3.01, FSPL≈181.696, losses=0, k_dB≈-228.599
        // C/N0 ≈ 55 + 3.01 - 181.696 + 228.599 = 104.913
        assert_approx_eq!(c_n0.as_f64(), 104.913, atol <= 0.1);
    }

    #[test]
    fn test_lumped_carrier_power_is_none() {
        use crate::receiver::GtReceiver;
        use crate::transmitter::EirpTransmitter;

        let tx = CommunicationSystem::eirp_only(EirpTransmitter {
            frequency: 29.0.ghz(),
            eirp: 55.0.db(),
        });
        let rx = CommunicationSystem::gt_only(GtReceiver {
            frequency: 29.0.ghz(),
            gt: 3.01.db(),
        });
        let p = tx
            .carrier_power(
                &rx,
                0.0.db(),
                Distance::kilometers(1000.0),
                Angle::ZERO,
                Angle::ZERO,
            )
            .unwrap();
        assert!(p.is_none());
    }

    #[test]
    fn test_unexpected_antenna_is_error() {
        use crate::antenna::ConstantAntenna;
        use crate::transmitter::EirpTransmitter;

        let tx = CommunicationSystem {
            antenna: Some(Antenna::Constant(ConstantAntenna { gain: 46.0.db() })),
            receiver: None,
            transmitter: Some(Transmitter::Eirp(EirpTransmitter {
                frequency: 29.0.ghz(),
                eirp: 55.0.db(),
            })),
        };
        let rx = rx_system();
        let err = tx
            .carrier_to_noise_density(
                &rx,
                0.0.db(),
                Distance::kilometers(1000.0),
                Angle::ZERO,
                Angle::ZERO,
            )
            .unwrap_err();
        assert_eq!(err, LinkBudgetError::UnexpectedAntenna);
    }

    #[test]
    fn test_missing_antenna_is_error() {
        use crate::transmitter::AmplifierTransmitter;

        let tx = CommunicationSystem {
            antenna: None,
            receiver: None,
            transmitter: Some(Transmitter::Amplifier(AmplifierTransmitter::new(
                29.0.ghz(),
                10.0,
                1.0.db(),
                0.0.db(),
            ))),
        };
        let rx = rx_system();
        let err = tx
            .carrier_to_noise_density(
                &rx,
                0.0.db(),
                Distance::kilometers(1000.0),
                Angle::ZERO,
                Angle::ZERO,
            )
            .unwrap_err();
        assert_eq!(err, LinkBudgetError::MissingAntenna);
    }

    #[test]
    fn test_frequency_mismatch_is_error() {
        use crate::receiver::GtReceiver;
        use crate::transmitter::EirpTransmitter;

        let tx = CommunicationSystem::eirp_only(EirpTransmitter {
            frequency: 29.0.ghz(),
            eirp: 55.0.db(),
        });
        let rx = CommunicationSystem::gt_only(GtReceiver {
            frequency: 30.0.ghz(),
            gt: 3.01.db(),
        });
        let err = tx
            .carrier_to_noise_density(
                &rx,
                0.0.db(),
                Distance::kilometers(1000.0),
                Angle::ZERO,
                Angle::ZERO,
            )
            .unwrap_err();
        assert!(matches!(err, LinkBudgetError::FrequencyMismatch { .. }));
    }

    #[test]
    fn test_missing_transmitter_is_error() {
        let tx = CommunicationSystem {
            antenna: None,
            receiver: None,
            transmitter: None,
        };
        let rx = rx_system();
        let err = tx
            .carrier_to_noise_density(
                &rx,
                0.0.db(),
                Distance::kilometers(1000.0),
                Angle::ZERO,
                Angle::ZERO,
            )
            .unwrap_err();
        assert_eq!(err, LinkBudgetError::MissingTransmitter);
    }

    #[test]
    fn test_missing_receiver_is_error() {
        let tx = tx_system();
        let rx = CommunicationSystem {
            antenna: None,
            receiver: None,
            transmitter: None,
        };
        let err = tx
            .carrier_to_noise_density(
                &rx,
                0.0.db(),
                Distance::kilometers(1000.0),
                Angle::ZERO,
                Angle::ZERO,
            )
            .unwrap_err();
        assert_eq!(err, LinkBudgetError::MissingReceiver);
    }

    #[test]
    fn test_noise_power_missing_receiver_is_error() {
        let sys = CommunicationSystem {
            antenna: None,
            receiver: None,
            transmitter: None,
        };
        let err = sys.noise_power(1e6).unwrap_err();
        assert_eq!(err, LinkBudgetError::MissingReceiver);
    }

    #[test]
    fn test_frequency_mismatch_carrier_power_is_error() {
        use crate::receiver::GtReceiver;
        use crate::transmitter::EirpTransmitter;

        let tx = CommunicationSystem::eirp_only(EirpTransmitter {
            frequency: 29.0.ghz(),
            eirp: 55.0.db(),
        });
        let rx = CommunicationSystem::gt_only(GtReceiver {
            frequency: 30.0.ghz(),
            gt: 3.01.db(),
        });
        let err = tx
            .carrier_power(
                &rx,
                0.0.db(),
                Distance::kilometers(1000.0),
                Angle::ZERO,
                Angle::ZERO,
            )
            .unwrap_err();
        assert!(matches!(err, LinkBudgetError::FrequencyMismatch { .. }));
    }

    #[test]
    fn test_amplifier_with_constructor() {
        let antenna = Antenna::Constant(ConstantAntenna { gain: 46.0.db() });
        let tx = AmplifierTransmitter::new(29.0.ghz(), 10.0, 1.0.db(), 0.0.db());
        let sys = CommunicationSystem::amplifier_with(antenna, tx);
        assert!(sys.antenna.is_some());
        assert!(sys.transmitter.is_some());
        assert!(sys.receiver.is_none());
    }

    #[test]
    fn test_receiver_with_constructor() {
        let antenna = Antenna::Constant(ConstantAntenna { gain: 30.0.db() });
        let rx = Receiver::NoiseTemperature(NoiseTempReceiver {
            frequency: 29.0.ghz(),
            system_noise_temperature: 500.0,
        });
        let sys = CommunicationSystem::receiver_with(antenna, rx);
        assert!(sys.antenna.is_some());
        assert!(sys.receiver.is_some());
        assert!(sys.transmitter.is_none());
    }

    #[test]
    fn test_carrier_power_missing_transmitter() {
        let tx = CommunicationSystem {
            antenna: None,
            receiver: None,
            transmitter: None,
        };
        let rx = rx_system();
        let err = tx
            .carrier_power(
                &rx,
                0.0.db(),
                Distance::kilometers(1000.0),
                Angle::ZERO,
                Angle::ZERO,
            )
            .unwrap_err();
        assert_eq!(err, LinkBudgetError::MissingTransmitter);
    }

    #[test]
    fn test_carrier_power_missing_receiver() {
        let tx = tx_system();
        let rx = CommunicationSystem {
            antenna: None,
            receiver: None,
            transmitter: None,
        };
        let err = tx
            .carrier_power(
                &rx,
                0.0.db(),
                Distance::kilometers(1000.0),
                Angle::ZERO,
                Angle::ZERO,
            )
            .unwrap_err();
        assert_eq!(err, LinkBudgetError::MissingReceiver);
    }

    #[test]
    fn test_carrier_power_missing_antenna_on_rx() {
        // Component-tier receiver (NoiseTemperature) with no antenna should fail.
        let tx = tx_system();
        let rx = CommunicationSystem {
            antenna: None,
            receiver: Some(Receiver::NoiseTemperature(NoiseTempReceiver {
                frequency: 29.0.ghz(),
                system_noise_temperature: 500.0,
            })),
            transmitter: None,
        };
        let err = tx
            .carrier_power(
                &rx,
                0.0.db(),
                Distance::kilometers(1000.0),
                Angle::ZERO,
                Angle::ZERO,
            )
            .unwrap_err();
        assert_eq!(err, LinkBudgetError::MissingAntenna);
    }

    #[test]
    fn test_eirp_at_missing_transmitter() {
        let sys = CommunicationSystem {
            antenna: None,
            receiver: None,
            transmitter: None,
        };
        let err = sys.eirp_at(Angle::ZERO).unwrap_err();
        assert_eq!(err, LinkBudgetError::MissingTransmitter);
    }

    #[test]
    fn test_eirp_at_lumped_returns_stored() {
        use crate::transmitter::EirpTransmitter;

        let sys = CommunicationSystem::eirp_only(EirpTransmitter {
            frequency: 29.0.ghz(),
            eirp: 55.0.db(),
        });
        let e = sys.eirp_at(Angle::ZERO).unwrap();
        assert_approx_eq!(e.as_f64(), 55.0, atol <= 1e-10);
        // Angle is ignored for the lumped Eirp variant
        let e_off = sys.eirp_at(Angle::radians(1.0)).unwrap();
        assert_approx_eq!(e_off.as_f64(), 55.0, atol <= 1e-10);
    }

    #[test]
    fn test_gt_at_missing_receiver() {
        let sys = CommunicationSystem {
            antenna: None,
            receiver: None,
            transmitter: None,
        };
        let err = sys.gt_at(Angle::ZERO).unwrap_err();
        assert_eq!(err, LinkBudgetError::MissingReceiver);
    }

    #[test]
    fn test_gt_at_lumped_returns_stored() {
        use crate::receiver::GtReceiver;

        let sys = CommunicationSystem::gt_only(GtReceiver {
            frequency: 29.0.ghz(),
            gt: 3.01.db(),
        });
        let g = sys.gt_at(Angle::ZERO).unwrap();
        assert_approx_eq!(g.as_f64(), 3.01, atol <= 1e-10);
    }

    #[test]
    fn test_tx_rx_frequency_accessors() {
        let tx = tx_system();
        let rx = rx_system();
        let empty = CommunicationSystem {
            antenna: None,
            receiver: None,
            transmitter: None,
        };
        assert_eq!(tx.tx_frequency().unwrap().to_hertz(), 29e9);
        assert!(tx.rx_frequency().is_none());
        assert_eq!(rx.rx_frequency().unwrap().to_hertz(), 29e9);
        assert!(rx.tx_frequency().is_none());
        assert!(empty.tx_frequency().is_none());
        assert!(empty.rx_frequency().is_none());
    }

    #[test]
    fn test_rx_gt_missing_antenna_for_component_receiver() {
        use crate::transmitter::EirpTransmitter;

        // Component-tier receiver (NoiseTemperature) with no antenna against
        // a lumped TX. The carrier_to_noise_density path goes through rx_gt
        // and should report MissingAntenna.
        let tx = CommunicationSystem::eirp_only(EirpTransmitter {
            frequency: 29.0.ghz(),
            eirp: 55.0.db(),
        });
        let rx = CommunicationSystem {
            antenna: None,
            receiver: Some(Receiver::NoiseTemperature(NoiseTempReceiver {
                frequency: 29.0.ghz(),
                system_noise_temperature: 500.0,
            })),
            transmitter: None,
        };
        let err = tx
            .carrier_to_noise_density(
                &rx,
                0.0.db(),
                Distance::kilometers(1000.0),
                Angle::ZERO,
                Angle::ZERO,
            )
            .unwrap_err();
        assert_eq!(err, LinkBudgetError::MissingAntenna);
    }
}
