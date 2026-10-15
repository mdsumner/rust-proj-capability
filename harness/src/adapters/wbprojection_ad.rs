//! wbprojection 0.3.3 (Whitebox): pure Rust, hand-built EPSG registry,
//! PROJ strings and WKT, CRS-to-CRS always through a WGS84 hub.

use wbprojection::Crs;

use crate::fixture::{authority, is_projjson, is_proj_string, is_wkt, Fixture};
use crate::judge::Outcome;
use crate::util;

fn build(def: &str) -> Result<Crs, String> {
    if util::is_crs84(def) {
        return Crs::from_epsg(4326).map_err(|e| e.to_string());
    }
    if is_proj_string(def) {
        return wbprojection::from_proj_string(def).map_err(|e| e.to_string());
    }
    if is_wkt(def) {
        return Crs::from_wkt(def).map_err(|e| e.to_string());
    }
    if is_projjson(def) {
        return Err("no PROJJSON input".into());
    }
    match authority(def) {
        // ESRI 54008/54009/54030 live in the same numeric registry
        Some((a, code)) if a == "EPSG" || a == "ESRI" => {
            let n: u32 = code.parse().map_err(|_| "bad code".to_string())?;
            Crs::from_epsg(n).map_err(|e| e.to_string())
        }
        Some((a, _)) => Err(format!("{a} codes not supported")),
        None => Err("unrecognised definition".into()),
    }
}

pub fn run(fx: &Fixture) -> Outcome {
    let (src, dst) = (fx.src.as_str(), fx.dst());
    let mut o = Outcome { fed_src: Some(src.into()), fed_dst: Some(dst.into()), ..Default::default() };
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
    o.resolved = true;
    let pt = util::src_point_deg(fx);
    if util::has_t(&pt) {
        o.notes.push("epoch not passed (transform_to_3d has no t)".into());
    }
    match conv(&s, &d, &pt) {
        Ok(v) => {
            o.executed = true;
            o.fwd = Some(util::deg_to_dst_native(fx, v));
        }
        Err(e) => {
            o.exec_error = Some(e);
            return o;
        }
    }
    if let Some(f) = util::dst_point_deg(fx) {
        match conv(&d, &s, &f) {
            Ok(v) => o.inv = Some(util::deg_to_src_native(fx, v)),
            Err(e) => o.inv_error = Some(e),
        }
    }
    o
}

fn conv(a: &Crs, b: &Crs, p: &[f64]) -> Result<Vec<f64>, String> {
    if util::has_z(p) {
        let (x, y, z) = a.transform_to_3d(p[0], p[1], p[2], b).map_err(|e| e.to_string())?;
        Ok(vec![x, y, z])
    } else {
        let (x, y) = a.transform_to(p[0], p[1], b).map_err(|e| e.to_string())?;
        Ok(vec![x, y])
    }
}
