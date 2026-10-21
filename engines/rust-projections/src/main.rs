//! projections-engines: every projection in suites/projections/reference.json
//! through every Rust engine, one JSON line per (projection, engine).
//!
//!   cargo run --release -- ../../suites/projections/reference.json
//!   cargo run --release -- -e proj4rs-main -p ortho ../../suites/projections/reference.json
//!   cargo run --release -- --list
//!
//! Each line carries the engine's forward result for every point and its
//! inverse of PROJ's forward result, or null where it gave none. Nothing is
//! judged here: report/projections.py judges every engine, Rust or JS, with
//! the same code.

mod engines;

use std::io::Write;
use std::panic::{catch_unwind, AssertUnwindSafe};

use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
struct Reference {
    environment: Value,
    projections: Vec<Projection>,
}

#[derive(Deserialize)]
pub struct Projection {
    pub id: String,
    pub proj: String,
    pub geographic: String,
    pub points: Vec<[f64; 2]>,
    #[serde(default)]
    pub fwd: Vec<Option<[f64; 2]>>,
}

/// A prepared lon/lat <-> projected transform. Geographic coordinates are degrees.
pub trait Prepared {
    fn fwd(&self, lon: f64, lat: f64) -> Option<[f64; 2]>;
    fn inv(&self, x: f64, y: f64) -> Option<[f64; 2]>;
}

pub struct Engine {
    pub name: &'static str,
    pub version: &'static str,
    pub prepare: fn(&Projection) -> Result<Box<dyn Prepared>, String>,
}

fn usage() -> ! {
    eprintln!("usage: projections-engines [--engine NAME]... [--projection ID]... [--list] [REFERENCE.json]");
    std::process::exit(2)
}

fn main() {
    let mut engines: Vec<String> = Vec::new();
    let mut only: Vec<String> = Vec::new();
    let mut path = String::from("suites/projections/reference.json");
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--engine" | "-e" => engines.push(args.next().unwrap_or_else(|| usage())),
            "--projection" | "-p" => only.push(args.next().unwrap_or_else(|| usage())),
            "--list" => {
                for e in engines::REGISTRY {
                    println!("{:16} {}", e.name, e.version);
                }
                return;
            }
            "-h" | "--help" => usage(),
            s if s.starts_with('-') => usage(),
            s => path = s.to_string(),
        }
    }
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("cannot read {path}: {e}");
        std::process::exit(1)
    });
    let reference: Reference = serde_json::from_str(&text).unwrap_or_else(|e| {
        eprintln!("cannot parse {path}: {e}");
        std::process::exit(1)
    });
    // reference.json schema this binary reads (suites/projections/README.md)
    let schema = reference.environment.get("schema_version").and_then(Value::as_u64);
    if schema != Some(1) {
        eprintln!("{path}: schema_version {schema:?}, this binary reads 1; regenerate with oracle/projections.py");
        std::process::exit(1)
    }

    // keep library panics out of the JSON stream
    std::panic::set_hook(Box::new(|_| {}));

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for e in engines::REGISTRY {
        if !engines.is_empty() && !engines.iter().any(|n| n == e.name) {
            continue;
        }
        for p in &reference.projections {
            if !only.is_empty() && !only.contains(&p.id) {
                continue;
            }
            let line = run(e, p);
            writeln!(out, "{line}").unwrap();
        }
    }
}

fn run(e: &Engine, p: &Projection) -> Value {
    let base = json!({"projection": p.id, "engine": e.name, "engine_version": e.version});
    let prepared = match catch_unwind(AssertUnwindSafe(|| (e.prepare)(p))) {
        Ok(Ok(t)) => t,
        Ok(Err(err)) => return merge(base, json!({"supported": false, "error": ascii(&err)})),
        Err(_) => return merge(base, json!({"supported": false, "error": "panic while preparing"})),
    };
    let mut panics = 0;
    let mut guard = |f: &dyn Fn() -> Option<[f64; 2]>| match catch_unwind(AssertUnwindSafe(f)) {
        Ok(v) => v.filter(|v| v[0].is_finite() && v[1].is_finite()),
        Err(_) => {
            panics += 1;
            None
        }
    };
    let fwd: Vec<Option<[f64; 2]>> = p.points.iter().map(|[lon, lat]| guard(&|| prepared.fwd(*lon, *lat))).collect();
    let inv: Vec<Option<[f64; 2]>> =
        p.fwd.iter().map(|xy| xy.and_then(|[x, y]| guard(&|| prepared.inv(x, y)))).collect();
    let mut v = merge(base, json!({"supported": true, "fwd": fwd, "inv": inv}));
    if panics > 0 {
        v["panics"] = json!(panics);
    }
    v
}

fn merge(mut a: Value, b: Value) -> Value {
    if let (Some(a), Value::Object(b)) = (a.as_object_mut(), b) {
        a.extend(b);
    }
    a
}

/// Library messages can carry non-ASCII (degree signs); keep the output ASCII.
fn ascii(s: &str) -> String {
    s.chars().map(|c| if c.is_ascii() { c } else { '?' }).collect()
}
