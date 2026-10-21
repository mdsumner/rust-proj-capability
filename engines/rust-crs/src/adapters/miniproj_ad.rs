//! miniproj 0.10.4: EPSG-code projections generated at build time from an
//! EPSG SQL dump. No CRS-to-CRS, no datum step: the harness chains
//! src.projected_to_deg -> dst.deg_to_projected, which is all a caller can do.

use miniproj::{get_projection, Projection};

use crate::fixture::{authority, Fixture};
use crate::judge::Outcome;
use crate::util;

fn lookup(def: &str) -> Result<&'static dyn Projection, String> {
    let code = if util::is_crs84(def) {
        4326
    } else {
        let Some((auth, code)) = authority(def) else {
            return Err("EPSG codes only (no text input)".into());
        };
        if auth != "EPSG" {
            return Err(format!("{auth} codes not supported (EPSG only)"));
        }
        code.parse::<u32>().map_err(|_| "bad code".to_string())?
    };
    get_projection(code).ok_or_else(|| format!("EPSG:{code} unknown or its method is not implemented"))
}

pub fn run(fx: &Fixture) -> Outcome {
    let s = match lookup(&fx.src) {
        Ok(p) => p,
        Err(e) => return Outcome::unresolved(format!("src: {e}")),
    };
    let d = match lookup(fx.dst()) {
        Ok(p) => p,
        Err(e) => return Outcome::unresolved(format!("dst: {e}")),
    };
    let mut o = Outcome { resolved: true, fed_src: Some(fx.src.clone()), fed_dst: Some(fx.dst().into()), ..Default::default() };
    o.notes.push("no datum step: src and dst ellipsoid frames chained as if identical".into());
    let pt = util::src_point_deg(fx);
    let (lon, lat) = s.projected_to_deg(pt[0], pt[1]);
    let (x, y) = d.deg_to_projected(lon, lat);
    o.executed = true;
    o.fwd = Some(util::deg_to_dst_native(fx, vec![x, y]));
    if let Some(f) = util::dst_point_deg(fx) {
        let (lon, lat) = d.projected_to_deg(f[0], f[1]);
        let (x, y) = s.deg_to_projected(lon, lat);
        o.inv = Some(util::deg_to_src_native(fx, vec![x, y]));
    }
    o
}
