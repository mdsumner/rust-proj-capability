//! proj4rs 0.2.1 (with proj4wkt 0.1.1 for WKT), the port tier.
//!
//! `proj4rs`: the definition as given. PROJ strings go straight in, WKT goes
//! through proj4wkt, authority codes and PROJJSON are not resolvable (the
//! crate's own EPSG path is the crs-definitions candidate).
//! `proj4rs+bound`: the wrapper column. proj4rs is fed the bound-to-WGS84
//! PROJ strings PROJ exported (oracle), as rwarp-wasm would be.

use proj4rs::transform::transform;
use proj4rs::Proj;

use crate::fixture::{authority, is_projjson, is_proj_string, is_wkt, Fixture};
use crate::judge::Outcome;
use crate::util;

/// Definition -> PROJ string proj4rs can parse.
fn to_projstring(def: &str) -> Result<String, String> {
    if util::is_crs84(def) {
        return Ok(util::HUB.to_string());
    }
    if is_proj_string(def) {
        return Ok(def.to_string());
    }
    if is_wkt(def) {
        return proj4wkt::wkt_to_projstring(def).map_err(|e| format!("proj4wkt: {e}"));
    }
    if is_projjson(def) {
        return Err("no PROJJSON input".into());
    }
    if authority(def).is_some() {
        return Err("no authority lookup (proj strings only)".into());
    }
    Err("unrecognised definition".into())
}

pub fn run(fx: &Fixture) -> Outcome {
    let src = match to_projstring(&fx.src) {
        Ok(s) => s,
        Err(e) => return Outcome::unresolved(format!("src: {e}")),
    };
    let dst = match to_projstring(fx.dst()) {
        Ok(s) => s,
        Err(e) => return Outcome::unresolved(format!("dst: {e}")),
    };
    execute(fx, &src, &dst)
}

pub fn run_bound(fx: &Fixture) -> Outcome {
    let src = if util::is_crs84(&fx.src) { Some(util::HUB) } else { fx.src_bound() };
    let (Some(src), Some(dst)) = (src, fx.dst_bound()) else {
        return Outcome::unresolved("PROJ produced no bound export for this CRS");
    };
    execute(fx, src, dst)
}

/// Run proj4rs on two PROJ strings; shared with the crs-definitions candidate.
pub fn execute(fx: &Fixture, src: &str, dst: &str) -> Outcome {
    let mut o = Outcome { fed_src: Some(src.into()), fed_dst: Some(dst.into()), ..Default::default() };
    let ps = match Proj::from_proj_string(src) {
        Ok(p) => p,
        Err(e) => {
            o.resolve_error = Some(format!("src: {e}"));
            return o;
        }
    };
    let pd = match Proj::from_proj_string(dst) {
        Ok(p) => p,
        Err(e) => {
            o.resolve_error = Some(format!("dst: {e}"));
            return o;
        }
    };
    o.resolved = true;
    let pt = util::src_point_deg(fx);
    match go(&ps, &pd, &pt) {
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
        match go(&pd, &ps, &f) {
            Ok(v) => o.inv = Some(util::deg_to_src_native(fx, util::like(&f, v))),
            Err(e) => o.inv_error = Some(e),
        }
    }
    o
}

/// proj4rs works in radians for geographic CRSs; the harness speaks degrees.
fn go(a: &Proj, b: &Proj, p: &[f64]) -> Result<Vec<f64>, String> {
    let mut c = (p[0], p[1], util::z_of(p));
    if a.is_latlong() {
        c.0 = c.0.to_radians();
        c.1 = c.1.to_radians();
    }
    transform(a, b, &mut c).map_err(|e| e.to_string())?;
    if b.is_latlong() {
        c.0 = c.0.to_degrees();
        c.1 = c.1.to_degrees();
    }
    Ok(vec![c.0, c.1, c.2])
}
