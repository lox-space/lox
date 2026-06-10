// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! End-to-end link budget for a LEO Earth-observation X-band downlink.
//!
//! A 500 km EO satellite downlinks payload data in the 8.025–8.4 GHz EESS
//! band to a 3.7 m ground station at 5° elevation (worst-case geometry).
//! The example builds both platforms as `CommsPayload` hardware
//! inventories, resolves their terminals into link endpoints, evaluates the
//! modulation-agnostic and DVB-S2-style modulated budgets, and checks the
//! downlink against the ITU RR Article 21.16 power flux density mask.
//!
//! Run from the workspace root:
//!
//! ```text
//! cargo run --example x_band_downlink -p lox-space
//! ```

use std::error::Error;

use lox_space::comms::antenna::Antenna;
use lox_space::comms::band::FrequencyRange;
use lox_space::comms::channel::{Channel, LinkDirection, Modulation};
use lox_space::comms::link_budget::{EnvironmentalLosses, LinkStats};
use lox_space::comms::payload::{CommsPayload, TxChain, TxPort};
use lox_space::comms::pfd::{PfdMask, power_flux_density};
use lox_space::comms::pointing::Pointing;
use lox_space::comms::receiver::{CascadeReceiver, NoiseStage, Receiver};
use lox_space::comms::transmitter::AmplifierTransmitter;
use lox_space::comms::utils::slant_range;
use lox_space::units::{
    AngleUnits, DecibelUnits, DistanceUnits, FrequencyUnits, PowerUnits, TemperatureUnits,
};

fn main() -> Result<(), Box<dyn Error>> {
    // ------------------------------------------------------------------
    // Mission geometry: 500 km orbit, tracked down to 5° elevation.
    // ------------------------------------------------------------------
    let eess_band = FrequencyRange::new(8.025.ghz(), 8.4.ghz())?;
    let carrier = 8.2.ghz();
    let elevation = 5.0.deg();
    let range = slant_range(elevation, 6371.0.km(), 500.0.km());
    println!(
        "Slant range at {}° elevation: {:.0} km",
        elevation.to_degrees(),
        range.to_kilometers()
    );

    // ------------------------------------------------------------------
    // Spacecraft: 0.25 m gimballed dish, 2 W X-band amplifier with 0.5 dB
    // output back-off, 0.8 dB feed run. The inventory is wired explicitly:
    // ports connect radios to antennas, terminals expose the operational
    // endpoints that link analysis addresses.
    // ------------------------------------------------------------------
    let mut spacecraft = CommsPayload::new();
    let dish = spacecraft.add_antenna("payload dish", Antenna::parabolic(0.25.m(), 0.6)?);
    let amplifier = spacecraft.add_transmitter(
        "x-band sspa",
        AmplifierTransmitter::new(eess_band, 2.0.w(), 0.5.db())?,
    );
    let tx_port = spacecraft.add_tx_port(
        TxPort::builder("tx feed", dish, amplifier)
            .feed_loss(0.8.db())
            .band(eess_band)
            .build()?,
    )?;
    let downlink_terminal =
        spacecraft.add_tx_terminal("x-band downlink", TxChain::Component(tx_port))?;

    // ------------------------------------------------------------------
    // Ground station: 3.7 m dish with an X-band front end described as a
    // Friis cascade (LNA → downconverter), 0.3 dB feed run, and 60 K
    // clear-sky antenna noise temperature at low elevation. The single-
    // chain convenience constructor wires the same structure in one call.
    // ------------------------------------------------------------------
    let lna = NoiseStage::new(35.0.db(), 50.0.k())?;
    let downconverter = NoiseStage::new(0.0.db(), 1540.0.k())?; // NF ≈ 8 dB
    let front_end = CascadeReceiver::new(
        eess_band,
        vec![lna, downconverter],
        0.5.db(), // demodulator loss
        0.5.db(), // implementation loss
    )?;
    let (ground_station, station_terminal) = CommsPayload::receiver_only(
        "3.7m station",
        Antenna::parabolic(3.7.m(), 0.6)?,
        Receiver::Cascade(front_end),
        0.3.db(),
        60.0.k(),
        Some(eess_band),
    )?;

    println!("\n{spacecraft}");
    println!("{ground_station}");

    // ------------------------------------------------------------------
    // Resolve the terminals into link endpoints and inspect the figures.
    // ------------------------------------------------------------------
    let tx = spacecraft.tx_endpoint(downlink_terminal)?;
    let rx = ground_station.rx_endpoint(station_terminal)?;

    // 2° residual pointing error on the spacecraft gimbal; the station
    // autotracks on boresight. Trajectory-driven analyses would pass
    // `Pointing::Direction` with line-of-sight vectors instead.
    let tx_pointing = Pointing::off_boresight(2.0.deg());
    let rx_pointing = Pointing::Boresight;

    println!("Effective TX band:   {}", tx.band());
    println!(
        "EIRP toward station: {:.1} dBW",
        tx.eirp_at(carrier, tx_pointing)?.as_f64()
    );
    println!(
        "Station G/T:         {:.1} dB/K",
        rx.gt_at(carrier, rx_pointing)?.as_f64()
    );
    println!(
        "Station T_sys:       {:.0} K (flange-referred)",
        rx.system_noise_temperature()
            .expect("component chain")
            .to_kelvin()
    );

    // ------------------------------------------------------------------
    // Atmospherics at X-band, 5° elevation, 99.9 % availability — static
    // values here; `lox-itur` computes them from the ITU-R P-series maps.
    // ------------------------------------------------------------------
    let losses = EnvironmentalLosses {
        rain: 1.2.db(),
        gaseous: 0.4.db(),
        scintillation: 0.3.db(),
        atmospheric: 0.0.db(),
        cloud: 0.1.db(),
        depolarization: 0.0.db(),
    };

    // ------------------------------------------------------------------
    // DVB-S2-style channel: QPSK 3/4 at 150 Msps → 225 Mbit/s net.
    // ------------------------------------------------------------------
    let channel = Channel {
        link_type: LinkDirection::Downlink,
        symbol_rate: 150.0.mhz(),
        required_eb_n0: 4.5.db(),
        margin: 3.0.db(),
        modulation: Modulation::Qpsk,
        roll_off: 0.25,
        fec: 0.75,
        chip_rate: None,
    };

    let link = LinkStats::for_link(
        &tx,
        &rx,
        carrier,
        channel.bandwidth(),
        range,
        losses,
        tx_pointing,
        rx_pointing,
        LinkDirection::Downlink,
    )?;
    let modulated = channel.apply(link);

    println!("\n--- Link budget at {} GHz ---", carrier.to_gigahertz());
    println!("EIRP:            {:>8.2} dBW", modulated.link.eirp.as_f64());
    println!("FSPL:            {:>8.2} dB", modulated.link.fspl.as_f64());
    println!(
        "Env. losses:     {:>8.2} dB",
        modulated.link.losses.total().as_f64()
    );
    println!("G/T:             {:>8.2} dB/K", modulated.link.gt.as_f64());
    println!(
        "C/N0:            {:>8.2} dB·Hz",
        modulated.link.c_n0.as_f64()
    );
    println!("C/N:             {:>8.2} dB", modulated.link.c_n.as_f64());
    println!("Es/N0:           {:>8.2} dB", modulated.es_n0.as_f64());
    println!("Eb/N0:           {:>8.2} dB", modulated.eb_n0.as_f64());
    println!(
        "Data rate:       {:>8.1} Mbit/s",
        channel.information_rate().to_megahertz()
    );
    println!("Link margin:     {:>8.2} dB", modulated.margin.as_f64());
    assert!(
        modulated.margin.as_f64() > 0.0,
        "the downlink should close at worst-case geometry"
    );

    // ------------------------------------------------------------------
    // Regulatory check: PFD on the ground vs. the RR Art. 21.16 mask
    // (−150 dBW/m²/4 kHz below 5° for the 8.025–8.4 GHz EESS allocation).
    // ------------------------------------------------------------------
    let pfd = power_flux_density(modulated.link.eirp, range, channel.bandwidth(), 4.0.khz());
    let mask = PfdMask::art_21_16((-150.0).db());
    let limit = mask.value_at(elevation);
    println!(
        "\nPFD at ground:   {:>8.2} dBW/m²/4kHz (limit {:.1})",
        pfd.as_f64(),
        limit.as_f64()
    );
    assert!(
        pfd.as_f64() <= limit.as_f64(),
        "the downlink should comply with RR Art. 21.16"
    );
    println!("RR Art. 21.16:   compliant");

    Ok(())
}
