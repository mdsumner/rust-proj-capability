//! xeibe-reproject 0.2.0: pure Rust, driven by EPSG's own data (an embedded
//! EPSG dataset as PROJJSON), choosing EPSG's operations per coordinate as
//! PROJ chooses them, and never a ballpark. Like the oracle (no grids
//! installed), grid-based operations are left out (`Grids::None`).

use xeibe_reproject::{AxisOrder, Crs, Grids, Transform};

use crate::fixture::Fixture;
use crate::judge::Outcome;
use crate::util;

fn build(def: &str) -> Result<Crs, String> {
    // the definition as given: EPSG codes in their spellings, OGC:CRS84, URNs, EPSG names,
    // PROJ strings (read as CRSs, +type=crs or not, as pyproj does), WKT, PROJJSON
    Crs::from_user_input(def).map_err(|e| e.to_string())
}

pub fn run(fx: &Fixture) -> Outcome {
    let (src, dst) = (fx.src.as_str(), fx.dst());
    let mut o = Outcome { fed_src: Some(src.into()), fed_dst: Some(dst.into()), ..Default::default() };
    if util::crs_typed(src).1 || util::crs_typed(dst).1 {
        o.notes.push("read as a CRS, as with +type=crs (a bare PROJ string is an operation, as in PROJ)".into());
    }
    let s = match build(src) {
        Ok(c) => c,
        Err(e) => {
            o.resolve_error = Some(format!("src: {e}"));
            return o;
        }
    };
    let d = match build(dst) {
        Ok(c) => c,
        Err(e) => {
            o.resolve_error = Some(format!("dst: {e}"));
            return o;
        }
    };
    let t = match Transform::with_grids(&s, &d, AxisOrder::EastFirst, Grids::None) {
        Ok(t) => t,
        Err(e) => {
            o.resolve_error = Some(format!("transform: {e}"));
            return o;
        }
    };
    o.resolved = true;
    // geographic coordinates in the CRS's own angular unit, as in PROJ
    let pt = fx.point();
    match conv(&t, &pt) {
        Ok(v) => {
            o.executed = true;
            o.operation = t.operation_for(pt[0], pt[1], pt.get(2).copied()).map(str::to_string);
            o.fwd = Some(v);
        }
        Err(e) => {
            o.exec_error = Some(e);
            return o;
        }
    }
    if let Some(f) = fx.oracle_fwd() {
        match t.inverse().map_err(|e| e.to_string()).and_then(|i| conv(&i, &f)) {
            Ok(v) => o.inv = Some(v),
            Err(e) => o.inv_error = Some(e),
        }
    }
    o
}

fn conv(t: &Transform, p: &[f64]) -> Result<Vec<f64>, String> {
    if util::has_t(p) {
        return Err("coordinate epochs are not supported".into());
    }
    let z = if util::has_z(p) { Some(p[2]) } else { None };
    let (x, y, z) = t.convert(p[0], p[1], z).map_err(|e| e.to_string())?;
    Ok(match z {
        Some(z) => vec![x, y, z],
        None => vec![x, y],
    })
}
