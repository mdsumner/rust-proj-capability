//! Shared adapter helpers: the WGS84 lon/lat hub and unit plumbing.

#![allow(dead_code)]

use crate::fixture::Fixture;

/// Lon/lat WGS84 as a PROJ string. Most fixtures start from OGC:CRS84;
/// candidates without an authority lookup are handed this instead, so the
/// shared starting point is not what is being tested.
pub const HUB: &str = "+proj=longlat +datum=WGS84 +no_defs";

pub fn is_crs84(def: &str) -> bool {
    matches!(def.trim(), "OGC:CRS84" | "CRS:84" | "urn:ogc:def:crs:OGC:1.3:CRS84")
}

/// Source point converted from native geographic units to degrees.
pub fn src_point_deg(fx: &Fixture) -> Vec<f64> {
    let mut p = fx.point();
    if fx.src_geographic() && p.len() >= 2 {
        let k = fx.src_deg_per_unit();
        p[0] *= k;
        p[1] *= k;
    }
    p
}

/// Oracle forward result as the input of an inverse, geographic in degrees.
pub fn dst_point_deg(fx: &Fixture) -> Option<Vec<f64>> {
    let mut p = fx.oracle_fwd()?;
    if fx.dst_geographic() && p.len() >= 2 {
        let k = fx.dst_deg_per_unit();
        p[0] *= k;
        p[1] *= k;
    }
    Some(p)
}

/// Degrees -> native units of the destination (grads for EPSG:4807).
pub fn deg_to_dst_native(fx: &Fixture, mut v: Vec<f64>) -> Vec<f64> {
    if fx.dst_geographic() && v.len() >= 2 {
        let k = fx.dst_deg_per_unit();
        v[0] /= k;
        v[1] /= k;
    }
    v
}

pub fn deg_to_src_native(fx: &Fixture, mut v: Vec<f64>) -> Vec<f64> {
    if fx.src_geographic() && v.len() >= 2 {
        let k = fx.src_deg_per_unit();
        v[0] /= k;
        v[1] /= k;
    }
    v
}

pub fn z_of(p: &[f64]) -> f64 {
    p.get(2).copied().unwrap_or(0.0)
}

pub fn has_z(p: &[f64]) -> bool {
    p.len() > 2
}

pub fn has_t(p: &[f64]) -> bool {
    p.len() > 3
}

/// Truncate a result to the dimension of the input (2D in, 2D out).
pub fn like(input: &[f64], mut v: Vec<f64>) -> Vec<f64> {
    v.truncate(input.len().max(2));
    v
}

/// libproj's crs_to_crs needs "+type=crs" on a PROJ string (pyproj appends it
/// silently). The proj adapters add it and say so in a note.
pub fn crs_typed(def: &str) -> (String, bool) {
    if crate::fixture::is_proj_string(def) && !def.contains("+type=crs") {
        (format!("{def} +type=crs"), true)
    } else {
        (def.to_string(), false)
    }
}
