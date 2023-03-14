use std::path::Path;
use std::{fs, io, io::prelude::*};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

pub fn main() -> Result<()> {
    let mut output = vec!["// AUTO-GENERATED DO NOT EDIT!".to_string(), "".to_string()];
    let mut in_data = false;
    let mut statement = Vec::new();
    for line in io::stdin().lock().lines() {
        let l = line?;
        if l.is_empty() {
            continue;
        };
        if !in_data && l.eq("\\begindata") {
            in_data = true;
        } else if in_data && l.eq("\\begintext") {
            in_data = false;
        } else if in_data {
            if l.trim().starts_with("BODY") && l.trim().ends_with(')') {
                output.push(format_statement(l.trim()));
            } else if l.trim().eq("BODY4_MAX_PHASE_DEGREE = 2") {
                output.push("const BODY4_MAX_PHASE_DEGREE: i32 = 2;".to_string());
            } else if l.trim().ends_with(')') {
                statement.push(l.trim().to_string());
                let s = statement.join(" ");
                statement.clear();
                output.push(format_statement(&s));
            } else {
                statement.push(l.trim().to_string());
            }
        } else {
            continue;
        }
    }

    let file = Path::new(file!());
    let out = file
        .parent()
        .unwrap()
        .join("../../../crates/lox_core/src/bodies/pck_constants.rs");

    fs::write(out, output.join("\n"))?;

    Ok(())
}

fn format_statement(statement: &str) -> String {
    let parts: Vec<_> = statement.splitn(2, '=').collect();
    let name = &parts[0].trim();
    let rest = &parts[1];
    let data: Vec<_> = rest
        .replace(['(', ')'], "")
        .replace('+', "")
        .replace('D', "e")
        .split_whitespace()
        .map(|x| {
            if !x.contains('.') {
                format!("{x}.0")
            } else {
                x.to_string()
            }
        })
        .collect();
    let n = data.len();
    let out = data.join(", ");
    format!("pub const {name}: [f64; {n}] = [{out}];")
}
