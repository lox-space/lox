// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Packager for the lox-itur-data.npz bundle.
//!
//! Usage: `cargo run -p lox-itur --bin pack -- <itur-wheel.whl> <out.npz>`
//!
//! Reads an upstream `itur` wheel from disk, unwraps each per-grid `.npz`
//! (a ZIP wrapping a single `arr_0.npy`), and writes the inner `.npy`
//! to the output bundle under its canonical `<rec>/<filename>.npy` key.
//! Emits a `manifest.json` with bundle version + upstream `itur` version.
//! Prints the SHA-256 of the output.

use std::env;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use std::process::ExitCode;

use zip::ZipArchive;
use zip::write::{SimpleFileOptions, ZipWriter};

const ITUR_DATA_PREFIX: &str = "itur/data";

const PROB_KEYS: [&str; 18] = [
    "01", "02", "03", "05", "1", "2", "3", "5", "10", "20", "30", "50", "60", "70", "80", "90",
    "95", "99",
];

/// (upstream path, canonical key inside the bundle).
fn grid_table() -> Vec<(String, String)> {
    let mut out = Vec::new();

    // P.1511 v2 (topography only; egm2008 is intentionally dropped — unused)
    for name in ["v2_lat", "v2_lon", "v2_topo"] {
        out.push((format!("1511/{name}.npz"), format!("1511/{name}.npy")));
    }

    // P.1510 v1 (annual + 12 monthly)
    out.push(("1510/v1_lat.npz".into(), "1510/v1_lat.npy".into()));
    out.push(("1510/v1_lon.npz".into(), "1510/v1_lon.npy".into()));
    out.push(("1510/v1_t_annual.npz".into(), "1510/v1_t_annual.npy".into()));
    for m in 1..=12u8 {
        out.push((
            format!("1510/v1_t_month{m:02}.npz"),
            format!("1510/v1_t_month{m:02}.npy"),
        ));
    }

    // P.453 v13 Nwet (18 prob levels)
    out.push(("453/v13_lat_n.npz".into(), "453/v13_lat_n.npy".into()));
    out.push(("453/v13_lon_n.npz".into(), "453/v13_lon_n.npy".into()));
    for p in PROB_KEYS {
        out.push((
            format!("453/v13_nwet_annual_{p}.npz"),
            format!("453/v13_nwet_annual_{p}.npy"),
        ));
    }

    // P.836 v6 (ρ, V, Vsch × 18 prob levels + topo subgrid)
    out.push(("836/v6_lat.npz".into(), "836/v6_lat.npy".into()));
    out.push(("836/v6_lon.npz".into(), "836/v6_lon.npy".into()));
    for prefix in ["rho", "v", "vsch"] {
        for p in PROB_KEYS {
            out.push((
                format!("836/v6_{prefix}_{p}.npz"),
                format!("836/v6_{prefix}_{p}.npy"),
            ));
        }
    }
    out.push(("836/v6_topolat.npz".into(), "836/v6_topolat.npy".into()));
    out.push(("836/v6_topolon.npz".into(), "836/v6_topolon.npy".into()));
    out.push((
        "836/v6_topo_0dot5.npz".into(),
        "836/v6_topo_0dot5.npy".into(),
    ));

    // P.837 v7 (r001 + 12 monthly mean T)
    out.push(("837/v7_lat_r001.npz".into(), "837/v7_lat_r001.npy".into()));
    out.push(("837/v7_lon_r001.npz".into(), "837/v7_lon_r001.npy".into()));
    out.push(("837/v7_r001.npz".into(), "837/v7_r001.npy".into()));
    out.push(("837/v7_lat_mt.npz".into(), "837/v7_lat_mt.npy".into()));
    out.push(("837/v7_lon_mt.npz".into(), "837/v7_lon_mt.npy".into()));
    for m in 1..=12u8 {
        out.push((
            format!("837/v7_mt_month{m:02}.npz"),
            format!("837/v7_mt_month{m:02}.npy"),
        ));
    }

    // P.839 v4 (rain isotherm 0°C height)
    out.push(("839/v4_esalat.npz".into(), "839/v4_esalat.npy".into()));
    out.push(("839/v4_esalon.npz".into(), "839/v4_esalon.npy".into()));
    out.push((
        "839/v4_esa0height.npz".into(),
        "839/v4_esa0height.npy".into(),
    ));

    // P.840 v7 (lred × 18 prob levels + m + sigma + pclw)
    out.push(("840/v7_lat.npz".into(), "840/v7_lat.npy".into()));
    out.push(("840/v7_lon.npz".into(), "840/v7_lon.npy".into()));
    for p in PROB_KEYS {
        out.push((
            format!("840/v7_lred_{p}.npz"),
            format!("840/v7_lred_{p}.npy"),
        ));
    }
    for name in ["m", "sigma", "pclw"] {
        out.push((format!("840/v7_{name}.npz"), format!("840/v7_{name}.npy")));
    }

    out
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: {} <itur-wheel.whl> <out.npz>", args[0]);
        return ExitCode::from(2);
    }
    let wheel_path = Path::new(&args[1]);
    let out_path = Path::new(&args[2]);
    match run(wheel_path, out_path) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run(wheel_path: &Path, out_path: &Path) -> std::io::Result<()> {
    let wheel_file = File::open(wheel_path)?;
    let mut wheel = ZipArchive::new(wheel_file).map_err(io_other)?;

    let upstream = parse_upstream_version(wheel_path);

    let out_file = File::create(out_path)?;
    let mut writer = ZipWriter::new(out_file);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    let table = grid_table();
    let mut grid_keys: Vec<String> = Vec::with_capacity(table.len());
    for (upstream_rel, bundle_key) in &table {
        let wheel_entry = format!("{ITUR_DATA_PREFIX}/{upstream_rel}");
        let inner_npy = unwrap_npz(&mut wheel, &wheel_entry)?;
        writer.start_file(bundle_key, opts).map_err(io_other)?;
        writer.write_all(&inner_npy)?;
        grid_keys.push(bundle_key.clone());
    }

    let manifest = lox_itur::manifest_for_packager(&upstream, grid_keys.clone());
    writer.start_file("manifest.json", opts).map_err(io_other)?;
    writer.write_all(&manifest)?;
    writer.finish().map_err(io_other)?;

    let sha = sha256_of_file(out_path)?;
    eprintln!(
        "packed {} grids from {upstream} → {} ({}, sha256: {sha})",
        table.len(),
        out_path.display(),
        human_size(std::fs::metadata(out_path)?.len()),
    );
    Ok(())
}

fn unwrap_npz(wheel: &mut ZipArchive<File>, entry_name: &str) -> std::io::Result<Vec<u8>> {
    let mut npz_bytes = Vec::new();
    wheel
        .by_name(entry_name)
        .map_err(|_| io_other_msg(format!("wheel missing entry {entry_name}")))?
        .read_to_end(&mut npz_bytes)?;
    let mut inner = ZipArchive::new(Cursor::new(npz_bytes)).map_err(io_other)?;
    let mut npy = Vec::new();
    inner
        .by_name("arr_0.npy")
        .map_err(|_| io_other_msg(format!("{entry_name} has no arr_0.npy")))?
        .read_to_end(&mut npy)?;
    Ok(npy)
}

fn parse_upstream_version(wheel_path: &Path) -> String {
    // wheel filename: itur-<version>-py2.py3-none-any.whl
    let stem = wheel_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("itur-unknown");
    let parts: Vec<&str> = stem.split('-').collect();
    if parts.len() >= 2 && parts[0] == "itur" {
        format!("itur-{}", parts[1])
    } else {
        "itur-unknown".to_owned()
    }
}

fn io_other<E: std::fmt::Display>(e: E) -> std::io::Error {
    io_other_msg(e.to_string())
}

fn io_other_msg(msg: String) -> std::io::Error {
    std::io::Error::other(msg)
}

fn sha256_of_file(path: &Path) -> std::io::Result<String> {
    use sha2::Digest;
    use std::io::BufReader;
    let mut hasher = sha2::Sha256::new();
    let mut reader = BufReader::new(File::open(path)?);
    let mut buf = [0u8; 8192];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        sha2::Digest::update(&mut hasher, &buf[..n]);
    }
    Ok(hex(&sha2::Digest::finalize(hasher)))
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        write!(s, "{b:02x}").unwrap();
    }
    s
}

fn human_size(n: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut x = n as f64;
    let mut i = 0;
    while x >= 1024.0 && i + 1 < UNITS.len() {
        x /= 1024.0;
        i += 1;
    }
    format!("{x:.1} {}", UNITS[i])
}
