// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

//! Build script that downloads ITU-R reference data from the itur PyPI package
//! and converts it to the binary grid format expected by lox-itur at runtime.

use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::{env, fs};

const WHEEL_URL: &str = "https://files.pythonhosted.org/packages/61/7b/e682678c0a6fcdd4529abc5f22324149cd7a725138465d76885a3a53c88f/itur-0.4.0-py2.py3-none-any.whl";
const COMPLETE_MARKER: &str = ".complete";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let data_dir = out_dir.join("itur-data");

    // Check if already converted
    if data_dir.join(COMPLETE_MARKER).exists() {
        return;
    }

    eprintln!("lox-itur: downloading ITU-R reference data from PyPI...");

    // Download wheel
    let wheel_path = out_dir.join("itur-0.4.0.whl");
    if !wheel_path.exists() {
        download_file(WHEEL_URL, &wheel_path);
    }

    eprintln!("lox-itur: converting data files...");
    fs::create_dir_all(&data_dir).unwrap();

    // Open wheel (it's a zip)
    let wheel_file = fs::File::open(&wheel_path).unwrap();
    let mut wheel = zip::ZipArchive::new(wheel_file).unwrap();

    // Extract and convert all needed data
    convert_p453(&mut wheel, &data_dir);
    convert_p1511(&mut wheel, &data_dir);
    convert_p1510(&mut wheel, &data_dir);
    convert_p836(&mut wheel, &data_dir);
    convert_p837(&mut wheel, &data_dir);
    convert_p839(&mut wheel, &data_dir);
    convert_p840(&mut wheel, &data_dir);

    // Write completion marker
    fs::write(data_dir.join(COMPLETE_MARKER), "ok").unwrap();
    eprintln!("lox-itur: data conversion complete.");
}

// ── HTTP download ───────────────────────────────────────────────────────────

fn download_file(url: &str, dest: &Path) {
    let resp = ureq::get(url)
        .call()
        .expect("failed to download itur wheel");
    let mut reader = resp.into_body().into_reader();
    let tmp = dest.with_extension("tmp");
    let mut file = fs::File::create(&tmp).unwrap();
    std::io::copy(&mut reader, &mut file).unwrap();
    fs::rename(&tmp, dest).unwrap();
}

// ── NPY parsing ─────────────────────────────────────────────────────────────

struct NpyArray {
    rows: usize,
    cols: usize,
    data: Vec<f64>,
}

fn parse_npy(bytes: &[u8]) -> NpyArray {
    // Verify magic
    assert_eq!(&bytes[..6], b"\x93NUMPY", "not a valid npy file");
    let _major = bytes[6];
    let _minor = bytes[7];
    let header_len = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
    let header = std::str::from_utf8(&bytes[10..10 + header_len]).unwrap();

    // Parse shape from header dict like: {'descr': '<f8', 'fortran_order': False, 'shape': (121, 241), }
    let (rows, cols) = parse_shape(header);

    let data_start = 10 + header_len;
    let expected_bytes = rows * cols * 8;
    assert!(
        bytes.len() >= data_start + expected_bytes,
        "npy data too short: need {} bytes from offset {}, have {}",
        expected_bytes,
        data_start,
        bytes.len()
    );

    let data: Vec<f64> = bytes[data_start..data_start + expected_bytes]
        .chunks_exact(8)
        .map(|c| f64::from_le_bytes(c.try_into().unwrap()))
        .collect();

    NpyArray { rows, cols, data }
}

fn parse_shape(header: &str) -> (usize, usize) {
    // Find 'shape': (rows, cols) or 'shape': (n,) for 1D
    let shape_start = header.find("'shape':").expect("no shape in npy header");
    let after = &header[shape_start..];
    let paren_start = after.find('(').unwrap();
    let paren_end = after.find(')').unwrap();
    let inner = &after[paren_start + 1..paren_end];
    let parts: Vec<&str> = inner
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    match parts.len() {
        1 => (1, parts[0].parse().unwrap()),
        2 => (parts[0].parse().unwrap(), parts[1].parse().unwrap()),
        _ => panic!("unexpected shape: {inner}"),
    }
}

// ── NPZ extraction from wheel ───────────────────────────────────────────────

fn read_npz_array(wheel: &mut zip::ZipArchive<fs::File>, wheel_path: &str) -> NpyArray {
    let mut npz_bytes = Vec::new();
    wheel
        .by_name(wheel_path)
        .unwrap_or_else(|_| panic!("file not found in wheel: {wheel_path}"))
        .read_to_end(&mut npz_bytes)
        .unwrap();

    // npz is a zip containing arr_0.npy
    let cursor = Cursor::new(npz_bytes);
    let mut npz = zip::ZipArchive::new(cursor).unwrap();
    let mut npy_bytes = Vec::new();
    npz.by_name("arr_0.npy")
        .expect("no arr_0.npy in npz")
        .read_to_end(&mut npy_bytes)
        .unwrap();

    parse_npy(&npy_bytes)
}

// ── Grid parameter extraction ───────────────────────────────────────────────

struct GridParams {
    lat_start: f64, // southernmost
    lat_step: f64,  // positive (S→N)
    lat_count: usize,
    lon_start: f64,
    lon_step: f64,
    lon_count: usize,
    needs_flip: bool, // whether data rows need to be reversed (N→S → S→N)
}

fn extract_grid_params(lat_arr: &NpyArray, lon_arr: &NpyArray) -> GridParams {
    // lat/lon arrays are 2D meshgrids; extract 1D vectors
    let lats: Vec<f64> = if lat_arr.rows > 1 {
        (0..lat_arr.rows)
            .map(|r| lat_arr.data[r * lat_arr.cols])
            .collect()
    } else {
        lat_arr.data.clone()
    };
    let lons: Vec<f64> = if lon_arr.rows > 1 {
        (0..lon_arr.cols).map(|c| lon_arr.data[c]).collect()
    } else {
        lon_arr.data.clone()
    };

    let lat_step_raw = if lats.len() > 1 {
        lats[1] - lats[0]
    } else {
        1.0
    };
    let lon_step = if lons.len() > 1 {
        (lons[1] - lons[0]).abs()
    } else {
        1.0
    };

    let needs_flip = lat_step_raw < 0.0;
    let (lat_start, lat_step) = if needs_flip {
        (*lats.last().unwrap(), lat_step_raw.abs())
    } else {
        (lats[0], lat_step_raw)
    };

    GridParams {
        lat_start,
        lat_step,
        lat_count: lats.len(),
        lon_start: lons[0],
        lon_step,
        lon_count: lons.len(),
        needs_flip,
    }
}

// ── Binary output ───────────────────────────────────────────────────────────

fn write_grid_bin(path: &Path, params: &GridParams, data: &NpyArray) {
    let mut raw = Vec::with_capacity(48 + data.data.len() * 8);

    // Header: 6 × f64
    raw.extend_from_slice(&params.lat_start.to_le_bytes());
    raw.extend_from_slice(&params.lat_step.to_le_bytes());
    raw.extend_from_slice(&(params.lat_count as f64).to_le_bytes());
    raw.extend_from_slice(&params.lon_start.to_le_bytes());
    raw.extend_from_slice(&params.lon_step.to_le_bytes());
    raw.extend_from_slice(&(params.lon_count as f64).to_le_bytes());

    // Data (flip rows if needed for S→N orientation)
    if params.needs_flip {
        for row in (0..data.rows).rev() {
            for col in 0..data.cols {
                raw.extend_from_slice(&data.data[row * data.cols + col].to_le_bytes());
            }
        }
    } else {
        for &v in &data.data {
            raw.extend_from_slice(&v.to_le_bytes());
        }
    }

    // Compress with zstd
    let compressed = zstd::encode_all(Cursor::new(&raw), 10).unwrap();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, compressed).unwrap();
}

/// Convert a single grid data file given pre-extracted grid params.
fn convert_grid(
    wheel: &mut zip::ZipArchive<fs::File>,
    data_dir: &Path,
    wheel_data_path: &str,
    out_name: &str,
    params: &GridParams,
) {
    let data = read_npz_array(wheel, wheel_data_path);
    let out_path = data_dir.join(format!("{out_name}.bin.zst"));
    write_grid_bin(&out_path, params, &data);
}

// ── Per-model converters ────────────────────────────────────────────────────

const ITUR_DATA: &str = "itur/data";

fn convert_p1511(wheel: &mut zip::ZipArchive<fs::File>, data_dir: &Path) {
    let lat = read_npz_array(wheel, &format!("{ITUR_DATA}/1511/v2_lat.npz"));
    let lon = read_npz_array(wheel, &format!("{ITUR_DATA}/1511/v2_lon.npz"));
    let params = extract_grid_params(&lat, &lon);

    for name in &["topo", "egm2008"] {
        convert_grid(
            wheel,
            data_dir,
            &format!("{ITUR_DATA}/1511/v2_{name}.npz"),
            &format!("1511/v2_{name}"),
            &params,
        );
    }
}

fn convert_p1510(wheel: &mut zip::ZipArchive<fs::File>, data_dir: &Path) {
    let lat = read_npz_array(wheel, &format!("{ITUR_DATA}/1510/v1_lat.npz"));
    let lon = read_npz_array(wheel, &format!("{ITUR_DATA}/1510/v1_lon.npz"));
    let params = extract_grid_params(&lat, &lon);

    convert_grid(
        wheel,
        data_dir,
        &format!("{ITUR_DATA}/1510/v1_t_annual.npz"),
        "1510/v1_t_annual",
        &params,
    );
    for month in 1..=12 {
        convert_grid(
            wheel,
            data_dir,
            &format!("{ITUR_DATA}/1510/v1_t_month{month:02}.npz"),
            &format!("1510/v1_t_month{month:02}"),
            &params,
        );
    }
}

fn convert_p453(wheel: &mut zip::ZipArchive<fs::File>, data_dir: &Path) {
    let lat = read_npz_array(wheel, &format!("{ITUR_DATA}/453/v13_lat_n.npz"));
    let lon = read_npz_array(wheel, &format!("{ITUR_DATA}/453/v13_lon_n.npz"));
    let params = extract_grid_params(&lat, &lon);

    let probs = [
        "01", "02", "03", "05", "1", "2", "3", "5", "10", "20", "30", "50", "60", "70", "80", "90",
        "95", "99",
    ];
    for p in &probs {
        convert_grid(
            wheel,
            data_dir,
            &format!("{ITUR_DATA}/453/v13_nwet_annual_{p}.npz"),
            &format!("453/v13_nwet_annual_{p}"),
            &params,
        );
    }
}

fn convert_p836(wheel: &mut zip::ZipArchive<fs::File>, data_dir: &Path) {
    let lat = read_npz_array(wheel, &format!("{ITUR_DATA}/836/v6_lat.npz"));
    let lon = read_npz_array(wheel, &format!("{ITUR_DATA}/836/v6_lon.npz"));
    let params = extract_grid_params(&lat, &lon);

    let probs = [
        "01", "02", "03", "05", "1", "2", "3", "5", "10", "20", "30", "50", "60", "70", "80", "90",
        "95", "99",
    ];
    for prefix in &["rho", "v", "vsch"] {
        for p in &probs {
            convert_grid(
                wheel,
                data_dir,
                &format!("{ITUR_DATA}/836/v6_{prefix}_{p}.npz"),
                &format!("836/v6_{prefix}_{p}"),
                &params,
            );
        }
    }

    // Topo sub-grid
    let topo_lat = read_npz_array(wheel, &format!("{ITUR_DATA}/836/v6_topolat.npz"));
    let topo_lon = read_npz_array(wheel, &format!("{ITUR_DATA}/836/v6_topolon.npz"));
    let topo_params = extract_grid_params(&topo_lat, &topo_lon);
    convert_grid(
        wheel,
        data_dir,
        &format!("{ITUR_DATA}/836/v6_topo_0dot5.npz"),
        "836/v6_topo_0dot5",
        &topo_params,
    );
}

fn convert_p837(wheel: &mut zip::ZipArchive<fs::File>, data_dir: &Path) {
    let lat_r = read_npz_array(wheel, &format!("{ITUR_DATA}/837/v7_lat_r001.npz"));
    let lon_r = read_npz_array(wheel, &format!("{ITUR_DATA}/837/v7_lon_r001.npz"));
    let params_r = extract_grid_params(&lat_r, &lon_r);
    convert_grid(
        wheel,
        data_dir,
        &format!("{ITUR_DATA}/837/v7_r001.npz"),
        "837/v7_r001",
        &params_r,
    );

    let lat_mt = read_npz_array(wheel, &format!("{ITUR_DATA}/837/v7_lat_mt.npz"));
    let lon_mt = read_npz_array(wheel, &format!("{ITUR_DATA}/837/v7_lon_mt.npz"));
    let params_mt = extract_grid_params(&lat_mt, &lon_mt);
    for month in 1..=12 {
        convert_grid(
            wheel,
            data_dir,
            &format!("{ITUR_DATA}/837/v7_mt_month{month:02}.npz"),
            &format!("837/v7_mt_month{month:02}"),
            &params_mt,
        );
    }
}

fn convert_p839(wheel: &mut zip::ZipArchive<fs::File>, data_dir: &Path) {
    let lat = read_npz_array(wheel, &format!("{ITUR_DATA}/839/v4_esalat.npz"));
    let lon = read_npz_array(wheel, &format!("{ITUR_DATA}/839/v4_esalon.npz"));
    let params = extract_grid_params(&lat, &lon);
    convert_grid(
        wheel,
        data_dir,
        &format!("{ITUR_DATA}/839/v4_esa0height.npz"),
        "839/v4_esa0height",
        &params,
    );
}

fn convert_p840(wheel: &mut zip::ZipArchive<fs::File>, data_dir: &Path) {
    let lat = read_npz_array(wheel, &format!("{ITUR_DATA}/840/v7_lat.npz"));
    let lon = read_npz_array(wheel, &format!("{ITUR_DATA}/840/v7_lon.npz"));
    let params = extract_grid_params(&lat, &lon);

    let probs = [
        "01", "02", "03", "05", "1", "2", "3", "5", "10", "20", "30", "50", "60", "70", "80", "90",
        "95", "99",
    ];
    for p in &probs {
        convert_grid(
            wheel,
            data_dir,
            &format!("{ITUR_DATA}/840/v7_lred_{p}.npz"),
            &format!("840/v7_lred_{p}"),
            &params,
        );
    }
    for name in &["m", "sigma", "pclw"] {
        convert_grid(
            wheel,
            data_dir,
            &format!("{ITUR_DATA}/840/v7_{name}.npz"),
            &format!("840/v7_{name}"),
            &params,
        );
    }
}
