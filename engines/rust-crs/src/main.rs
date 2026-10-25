//! capability-harness: the same fixtures through every candidate, one JSON
//! line per (fixture, candidate), judged against the PROJ oracle in
//! suites/crs/crs.json by code that belongs to none of them.
//!
//!   cargo run -- --candidate proj4rs ../fixtures/crs.json
//!   cargo run -- --candidate all ../fixtures/crs.json
//!   cargo run -- --list
//!
//! A candidate whose cargo feature is not built is skipped (one
//! {"skipped": true} line), not failed.

mod adapters;
mod fixture;
mod judge;
mod util;

use std::io::Write;
use std::panic::{catch_unwind, AssertUnwindSafe};

use fixture::FixtureFile;
use judge::{judge, Outcome};

fn usage() -> ! {
    eprintln!("usage: capability-harness [--candidate NAME|all]... [--fixture ID]... [--list] [FIXTURES.json]");
    std::process::exit(2)
}

fn main() {
    let mut candidates: Vec<String> = Vec::new();
    let mut only: Vec<String> = Vec::new();
    let mut path = String::from("suites/crs/crs.json");
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--candidate" | "-c" => candidates.push(args.next().unwrap_or_else(|| usage())),
            "--fixture" | "-f" => only.push(args.next().unwrap_or_else(|| usage())),
            "--list" => {
                for c in adapters::REGISTRY {
                    println!("{:16} {}", c.name, if c.run.is_some() { "built" } else { "not built" });
                }
                return;
            }
            "-h" | "--help" => usage(),
            s if s.starts_with('-') => usage(),
            s => path = s.to_string(),
        }
    }
    if candidates.is_empty() || candidates.iter().any(|c| c == "all") {
        candidates = adapters::REGISTRY.iter().map(|c| c.name.to_string()).collect();
    }

    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("cannot read {path}: {e}");
        std::process::exit(1)
    });
    let file: FixtureFile = serde_json::from_str(&text).unwrap_or_else(|e| {
        eprintln!("cannot parse {path}: {e}");
        std::process::exit(1)
    });

    // suites/crs/crs.json schema this harness reads (suites/crs/README.md, "Schema")
    let schema = file.environment.get("schema_version").and_then(|v| v.as_u64());
    if schema != Some(1) {
        eprintln!("{path}: schema_version {schema:?}, this harness reads 1; regenerate with oracle/crs.py");
        std::process::exit(1)
    }

    // keep library panics out of the JSON stream
    std::panic::set_hook(Box::new(|_| {}));

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for name in &candidates {
        let Some(entry) = adapters::REGISTRY.iter().find(|c| c.name == name) else {
            eprintln!("unknown candidate {name}; try --list");
            std::process::exit(2)
        };
        let Some(run) = entry.run else {
            let line = serde_json::json!({"candidate": name, "skipped": true, "reason": "cargo feature not built"});
            writeln!(out, "{line}").unwrap();
            continue;
        };
        for fx in &file.fixtures {
            if !only.is_empty() && !only.contains(&fx.id) {
                continue;
            }
            let o = match catch_unwind(AssertUnwindSafe(|| run(fx))) {
                Ok(o) => o,
                Err(p) => {
                    let msg = p
                        .downcast_ref::<String>()
                        .cloned()
                        .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()))
                        .unwrap_or_else(|| "panic".into());
                    Outcome { exec_error: Some(format!("panic: {msg}")), ..Default::default() }
                        .note("library panicked")
                }
            };
            let line = judge(fx, name, o, &file.fixtures);
            writeln!(out, "{}", ascii_json(&serde_json::to_string(&line).unwrap())).unwrap();
        }
    }
}

/// Escape non-ASCII as \uXXXX (library messages carry degree signs); output stays valid JSON.
fn ascii_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii() {
            out.push(c);
        } else {
            let mut buf = [0u16; 2];
            for u in c.encode_utf16(&mut buf) {
                out.push_str(&format!("\\u{:04x}", u));
            }
        }
    }
    out
}

/// E7 thread safety, checked by the compiler: `cargo test --no-run` fails if
/// any of these handle types is not Send + Sync. (proj::Proj is checked
/// separately in platform/send-sync.txt: it is neither.)
#[cfg(test)]
#[allow(dead_code)]
mod send_sync {
    fn ok<T: Send + Sync>() {}
    #[cfg(feature = "c-proj4rs")]
    fn proj4rs() {
        ok::<proj4rs::Proj>();
    }
    #[cfg(feature = "c-geodesy")]
    fn geodesy() {
        ok::<geodesy::prelude::Minimal>();
    }
    #[cfg(feature = "c-miniproj")]
    fn miniproj() {
        ok::<&'static dyn miniproj::Projection>();
    }
    #[cfg(feature = "c-projcore")]
    fn projcore() {
        ok::<proj_core::Transform>();
    }
    #[cfg(feature = "c-wbprojection")]
    fn wb() {
        ok::<wbprojection::Crs>();
    }
    #[cfg(feature = "c-terrio-crs")]
    fn terrio_crs() {
        ok::<terrio_crs::Transformer>();
    }
    #[cfg(feature = "c-proj-send-sync-probe")]
    fn proj() {
        ok::<proj::Proj>();
    }
}
