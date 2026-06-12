// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! End-to-end link budget for a LEO Earth-observation X-band downlink.
//!
//! A 500 km EO satellite downlinks payload data in the 8.025–8.4 GHz EESS
//! band to a 3.7 m ground station at 5° elevation (worst-case geometry).
//! The example builds both link ends as terminal values (`TxChain` and
//! `RxChain`), computes the modulation-agnostic and DVB-S2-style modulated
//! budgets, and checks the downlink against the ITU RR Article 21.16 power
//! flux density mask.
//!
//! Run from the workspace root:
//!
//! ```text
//! cargo run --example x_band_downlink -p lox-space
//! ```

use std::error::Error;

use lox_space::comms::FrequencyRange;
use lox_space::comms::antenna::Antenna;
use lox_space::comms::channel::{Channel, LinkDirection};
use lox_space::comms::link_budget::{Eirp, GOverT, LinkBudget, LinkConditions, PropagationLosses};
use lox_space::comms::modcod::dvb_s2;
use lox_space::comms::pfd::{PfdMask, power_flux_density};
use lox_space::comms::pointing::Pointing;
use lox_space::comms::receiver::CascadeReceiver;
use lox_space::comms::terminal::{RxChain, TxChain};
use lox_space::comms::transmitter::AmplifierTransmitter;
use lox_space::comms::utils::slant_range;
use lox_space::units::{
    AngleUnits, DecibelUnits, DistanceUnits, FrequencyUnits, PowerUnits, TemperatureUnits,
};

fn main() -> Result<(), Box<dyn Error>> {
    // ------------------------------------------------------------------
    // Mission geometry: 500 km orbit, tracked down to 5° elevation.
    // ------------------------------------------------------------------
    // The EESS allocation is a sub-band of IEEE X band; `FrequencyBand::X`
    // would give the full 8-12 GHz letter band instead.
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
    // output back-off, 0.8 dB feed run — one transmit chain.
    // ------------------------------------------------------------------
    let tx = TxChain::new(
        Antenna::parabolic(0.25.m(), 0.6)?,
        AmplifierTransmitter::new(2.0.w(), 0.5.db())?,
        0.8.db(),
        eess_band,
    )?;

    // ------------------------------------------------------------------
    // Ground station: 3.7 m dish with an X-band front end described as a
    // Friis cascade (LNA → downconverter), 0.3 dB feed run, and 60 K
    // clear-sky antenna noise temperature at low elevation.
    // ------------------------------------------------------------------
    let front_end = CascadeReceiver::builder()
        .stage(35.0.db(), 50.0.k()) // LNA
        .stage(0.0.db(), 1540.0.k()) // downconverter, NF ≈ 8 dB
        .demodulator_loss(0.5.db())
        .implementation_loss(0.5.db())
        .build()?;
    let rx = RxChain::new(
        Antenna::parabolic(3.7.m(), 0.6)?,
        front_end,
        0.3.db(),
        60.0.k(),
        eess_band,
    )?;

    // 2° residual pointing error on the spacecraft gimbal; the station
    // autotracks on boresight. Trajectory-driven analyses would pass
    // `Pointing::Direction` with line-of-sight vectors instead.
    let tx_pointing = Pointing::off_boresight(2.0.deg());
    let rx_pointing = Pointing::Boresight;

    println!("TX band:             {}", tx.band());
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
        rx.system_noise_temperature().to_kelvin()
    );

    // ------------------------------------------------------------------
    // Atmospherics at X-band, 5° elevation, 99.9 % availability — static
    // values here; `lox-itur` computes them from the ITU-R P-series maps.
    // ------------------------------------------------------------------
    let losses = PropagationLosses::builder()
        .rain(1.2.db())
        .gaseous(0.4.db())
        .scintillation(0.3.db())
        .cloud(0.1.db())
        .build()?;

    // ------------------------------------------------------------------
    // DVB-S2 QPSK 3/4 at 150 Msps; the MODCOD table carries the exact
    // Table 13 thresholds and coding chains (BBFRAME, BCH, LDPC, PLFRAME).
    // ------------------------------------------------------------------
    let channel = Channel::builder(150.0.mhz()).roll_off(0.25).build()?;
    let modcod = dvb_s2()
        .iter()
        .find(|mc| mc.mode().name() == "QPSK 3/4")
        .expect("table mode");
    let design_margin = 3.0.db();

    let conditions = LinkConditions::builder(carrier, range)
        .losses(losses)
        .tx_pointing(tx_pointing)
        .rx_pointing(rx_pointing)
        .direction(LinkDirection::Downlink)
        .build()?;
    let budget = LinkBudget::new(&tx, &rx, &conditions)?;
    let modulated = budget.modulate(&channel, modcod, design_margin);

    println!("\n--- Link budget at {} GHz ---", carrier.to_gigahertz());
    print!("{modulated}");
    println!(
        "Data rate:       {:>8.1} Mbit/s",
        modulated.information_rate().to_megahertz()
    );
    assert!(
        modulated.closes(),
        "the downlink should close at worst-case geometry"
    );

    // ------------------------------------------------------------------
    // ACM: the best DVB-S2 mode that closes at this Es/N0 with the same
    // design margin.
    // ------------------------------------------------------------------
    let best = budget
        .modulate_best(&channel, dvb_s2(), design_margin)
        .expect("a mode closes");
    println!(
        "\nBest ACM mode:   {} ({:.1} Mbit/s, +{:.0} %)",
        best.modcod.mode().name(),
        best.information_rate().to_megahertz(),
        (best.modcod.mode().info_bits_per_symbol() / modcod.mode().info_bits_per_symbol() - 1.0)
            * 100.0
    );

    // ------------------------------------------------------------------
    // Regulatory check: PFD on the ground vs. the RR Art. 21.16 mask
    // (−150 dBW/m²/4 kHz below 5° for the 8.025–8.4 GHz EESS allocation).
    // ------------------------------------------------------------------
    let pfd = power_flux_density(modulated.budget.eirp, range, channel.bandwidth(), 4.0.khz());
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
