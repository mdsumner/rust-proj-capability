//! geodesy 0.15 (Rust Geodesy), the rethink tier. Operators, not CRSs.
//!
//! `geodesy`: PROJ strings as given, through geodesy's own
//! `authoring::parse_proj` translation, composed as `inv src | dst`. That is
//! what the crate offers a CRS-shaped caller: it ignores +datum/+towgs84/
//! +units silently (default ellipsoid GRS80).
//! `geodesy+bound`: the wrapper column. The harness translates PROJ's bound
//! exports into a geodesy pipeline itself (ellipsoid, +towgs84 as
//! cart | helmert | inv cart through WGS84, +units as unitconvert): what a
//! thin CRS layer over geodesy's operators would have to supply.

use geodesy::authoring::parse_proj;
use geodesy::prelude::*;

use crate::fixture::{authority, is_projjson, is_proj_string, is_wkt, Fixture};
use crate::judge::Outcome;
use crate::util;

pub fn run(fx: &Fixture) -> Outcome {
    let side = |d: &str| -> Result<String, String> {
        if util::is_crs84(d) {
            return Ok(String::new());
        }
        if is_proj_string(d) {
            let g = parse_proj(d).map_err(|e| format!("parse_proj: {e}"))?;
            let g = g.trim().to_string();
            // geographic CRSs translate to a noop alias
            if matches!(g.split_whitespace().next(), Some("longlat" | "latlong" | "lonlat" | "latlon" | "noop")) {
                return Ok(String::new());
            }
            return Ok(g);
        }
        Err(if is_wkt(d) {
            "no WKT input".into()
        } else if is_projjson(d) {
            "no PROJJSON input".into()
        } else if authority(d).is_some() {
            "no authority lookup".into()
        } else {
            "unrecognised definition".into()
        })
    };
    let s = match side(&fx.src) {
        Ok(s) => s,
        Err(e) => return Outcome::unresolved(format!("src: {e}")),
    };
    let d = match side(fx.dst()) {
        Ok(s) => s,
        Err(e) => return Outcome::unresolved(format!("dst: {e}")),
    };
    let mut steps = Vec::new();
    if !s.is_empty() {
        steps.push(format!("inv {s}"));
    }
    if !d.is_empty() {
        steps.push(d);
    }
    execute(fx, steps, fx.src.clone(), fx.dst().to_string())
}

pub fn run_bound(fx: &Fixture) -> Outcome {
    let src = if util::is_crs84(&fx.src) { Some(util::HUB) } else { fx.src_bound() };
    let (Some(src), Some(dst)) = (src, fx.dst_bound()) else {
        return Outcome::unresolved("PROJ produced no bound export for this CRS");
    };
    let a = match Crs::parse(src) {
        Ok(c) => c,
        Err(e) => return Outcome::unresolved(format!("src: {e}")),
    };
    let b = match Crs::parse(dst) {
        Ok(c) => c,
        Err(e) => return Outcome::unresolved(format!("dst: {e}")),
    };
    let mut steps = Vec::new();
    if let Some(u) = &a.unitconvert {
        steps.push(format!("inv {u}"));
    }
    if let Some(p) = &a.proj {
        steps.push(format!("inv {p}"));
    }
    // datum: proj4 semantics, hub through WGS84 only when both sides carry one
    if let (Some(ta), Some(tb)) = (&a.towgs84, &b.towgs84) {
        if !a.null_grid && !b.null_grid && (ta != tb || a.ellps != b.ellps) {
            steps.push(format!("cart ellps={}", a.ellps));
            if let Some(h) = helmert(ta) {
                steps.push(h);
            }
            if let Some(h) = helmert(tb) {
                steps.push(format!("inv {h}"));
            }
            steps.push(format!("inv cart ellps={}", b.ellps));
        }
    }
    if let Some(p) = &b.proj {
        steps.push(p.clone());
    }
    if let Some(u) = &b.unitconvert {
        steps.push(u.clone());
    }
    execute(fx, steps, src.to_string(), dst.to_string())
}

fn execute(fx: &Fixture, steps: Vec<String>, fed_src: String, fed_dst: String) -> Outcome {
    let def = if steps.is_empty() { "noop".to_string() } else { steps.join(" | ") };
    let mut o = Outcome { fed_src: Some(fed_src), fed_dst: Some(fed_dst), ..Default::default() };
    o.operation = Some(def.clone());
    let mut ctx = Minimal::new();
    let op = match ctx.op(&def) {
        Ok(op) => op,
        Err(e) => {
            o.resolve_error = Some(format!("{e}"));
            return o;
        }
    };
    o.resolved = true;
    let pt = util::src_point_deg(fx);
    match apply(&ctx, op, Fwd, &pt, fx.src_geographic(), fx.dst_geographic()) {
        Ok(v) => {
            o.executed = true;
            o.fwd = Some(util::deg_to_dst_native(fx, util::like(&pt, v)));
        }
        Err(e) => {
            o.exec_error = Some(e);
            return o;
        }
    }
    if let Some(f) = util::dst_point_deg(fx) {
        match apply(&ctx, op, Inv, &f, fx.dst_geographic(), fx.src_geographic()) {
            Ok(v) => o.inv = Some(util::deg_to_src_native(fx, util::like(&f, v))),
            Err(e) => o.inv_error = Some(e),
        }
    }
    o
}

/// geodesy is radians in, radians out for angular coordinates; NaN marks a failed point.
fn apply(ctx: &Minimal, op: OpHandle, dir: Direction, p: &[f64], geog_in: bool, geog_out: bool) -> Result<Vec<f64>, String> {
    let (mut x, mut y) = (p[0], p[1]);
    if geog_in {
        x = x.to_radians();
        y = y.to_radians();
    }
    // t = 0 when there is no epoch: `cart` counts a point with a NaN in any
    // of its four components as failed, so geodesy's own Coor2D convention
    // (t = NaN) would report failure for a good point. Judge by x, y only.
    let t = p.get(3).copied().unwrap_or(0.0);
    let mut c = [Coor4D::raw(x, y, util::z_of(p), t)];
    ctx.apply(op, dir, &mut c).map_err(|e| e.to_string())?;
    let mut v = vec![c[0][0], c[0][1], c[0][2], c[0][3]];
    if v[0].is_nan() || v[1].is_nan() {
        return Err("point failed (NaN)".into());
    }
    if geog_out {
        v[0] = v[0].to_degrees();
        v[1] = v[1].to_degrees();
    }
    Ok(v)
}

fn helmert(t: &[f64]) -> Option<String> {
    if t.iter().all(|v| *v == 0.0) {
        return None;
    }
    let mut s = format!("helmert x={} y={} z={}", t[0], t[1], t[2]);
    if t.len() == 7 {
        s += &format!(" rx={} ry={} rz={} s={} convention=position_vector", t[3], t[4], t[5], t[6]);
    }
    Some(s)
}

/// A PROJ CRS string split into what a geodesy pipeline needs.
struct Crs {
    proj: Option<String>,
    ellps: String,
    towgs84: Option<Vec<f64>>,
    /// +nadgrids=@null: "lat/lon are already WGS84", no geocentric step
    null_grid: bool,
    unitconvert: Option<String>,
}

impl Crs {
    fn parse(def: &str) -> Result<Self, String> {
        let mut kv: Vec<(String, Option<String>)> = Vec::new();
        for tok in def.split_whitespace() {
            let tok = tok.trim_start_matches('+');
            match tok.split_once('=') {
                Some((k, v)) => kv.push((k.into(), Some(v.into()))),
                None => kv.push((tok.into(), None)),
            }
        }
        let get = |k: &str| kv.iter().find(|(a, _)| a == k).and_then(|(_, v)| v.clone());
        let proj = get("proj").ok_or("no +proj")?;
        let mut ellps = get("ellps");
        let mut towgs84: Option<Vec<f64>> = None;
        match get("datum").as_deref() {
            Some("WGS84") => {
                ellps.get_or_insert("WGS84".into());
                towgs84 = Some(vec![0.0; 3]);
            }
            Some("NAD83") => {
                ellps.get_or_insert("GRS80".into());
                towgs84 = Some(vec![0.0; 3]);
            }
            Some(d) => return Err(format!("+datum={d} needs a translation table (grids)")),
            None => {}
        }
        if let Some(t) = get("towgs84") {
            let v: Result<Vec<f64>, _> = t.split(',').map(str::parse::<f64>).collect();
            towgs84 = Some(v.map_err(|_| "bad +towgs84")?);
        }
        let mut null_grid = false;
        if let Some(g) = get("nadgrids") {
            if g != "@null" {
                return Err(format!("+nadgrids={g}: grids not translated"));
            }
            null_grid = true;
            towgs84 = Some(vec![0.0; 3]);
        }
        if let (Some(a), None) = (get("a"), &ellps) {
            let a: f64 = a.parse().map_err(|_| "bad +a")?;
            let rf = match (get("rf"), get("b")) {
                (Some(rf), _) => rf.parse::<f64>().map_err(|_| "bad +rf")?,
                (None, Some(b)) => {
                    let b: f64 = b.parse().map_err(|_| "bad +b")?;
                    if b == a { f64::INFINITY } else { a / (a - b) }
                }
                _ => f64::INFINITY,
            };
            ellps = Some(format!("{a},{rf}"));
        }
        if get("R").is_some() {
            return Err("+R sphere not translated".into());
        }
        if get("pm").is_some() {
            return Err("+pm not translated".into());
        }
        let ellps = ellps.unwrap_or_else(|| "GRS80".into());
        let unitconvert = match get("units").as_deref() {
            None | Some("m") => None,
            Some(u) => Some(format!("unitconvert xy_in=m xy_out={u}")),
        };
        let proj = match proj.as_str() {
            "longlat" | "latlong" | "lonlat" | "latlon" => None,
            "geocent" => return Err("geocentric CRS: not translated by this wrapper".into()),
            _ => {
                // hand the projection parameters to geodesy's own translator
                let keep: Vec<String> = kv
                    .iter()
                    .filter(|(k, _)| {
                        !matches!(k.as_str(), "datum" | "towgs84" | "nadgrids" | "units" | "no_defs" | "type" | "wktext" | "ellps" | "a" | "b" | "rf")
                    })
                    .map(|(k, v)| match v {
                        Some(v) => format!("+{k}={v}"),
                        None => format!("+{k}"),
                    })
                    .collect();
                let g = parse_proj(&keep.join(" ")).map_err(|e| format!("parse_proj: {e}"))?;
                Some(format!("{} ellps={ellps}", g.trim()))
            }
        };
        Ok(Crs { proj, ellps, towgs84, null_grid, unitconvert })
    }
}
