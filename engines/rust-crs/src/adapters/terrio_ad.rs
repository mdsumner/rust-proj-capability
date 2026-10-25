//! terrio 0.5.0 (opengeos): pure Rust, its own CRS engine aimed at matching
//! GDAL's choice of datum transformation; wbprojection is its fallback for
//! projections it does not implement. Built without default features, so no
//! network grid fetching.

use terrio::crs::{SpatialRef, Transformer};

use crate::fixture::{is_projjson, Fixture};
use crate::judge::Outcome;
use crate::util;

fn build(def: &str) -> Result<SpatialRef, String> {
    if is_projjson(def) {
        return Err("no PROJJSON input".into());
    }
    // the definition as given: EPSG/ESRI codes, URNs, OGC:CRS84, WKT, PROJ strings
    SpatialRef::parse(def).map_err(|e| e.to_string())
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
    let fwd_t = match Transformer::new(&s, &d) {
        Ok(t) => t,
        Err(e) => {
            o.resolve_error = Some(format!("transformer: {e}"));
            return o;
        }
    };
    o.resolved = true;
    // terrio takes and returns geographic coordinates in the CRS's own angular
    // unit, as GDAL does (grads for EPSG:4807), so no degree conversion here
    let pt = fx.point();
    match conv(&fwd_t, &pt) {
        Ok(v) => {
            o.executed = true;
            o.fwd = Some(v);
        }
        Err(e) => {
            o.exec_error = Some(e);
            return o;
        }
    }
    if let Some(f) = fx.oracle_fwd() {
        match Transformer::new(&d, &s).map_err(|e| e.to_string()).and_then(|t| conv(&t, &f)) {
            Ok(v) => o.inv = Some(v),
            Err(e) => o.inv_error = Some(e),
        }
    }
    o
}

fn conv(t: &Transformer, p: &[f64]) -> Result<Vec<f64>, String> {
    if util::has_t(p) {
        let (x, y, z) = t.transform_4d(p[0], p[1], p[2], p[3]).map_err(|e| e.to_string())?;
        Ok(vec![x, y, z])
    } else if util::has_z(p) {
        let (x, y, z) = t.transform_3d(p[0], p[1], p[2]).map_err(|e| e.to_string())?;
        Ok(vec![x, y, z])
    } else {
        let (x, y) = t.transform(p[0], p[1]).map_err(|e| e.to_string())?;
        Ok(vec![x, y])
    }
}
