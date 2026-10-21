//! proj-core 0.11.0 + proj-wkt 0.11.0 (roteiro-gis/proj-rust): pure Rust
//! with an embedded EPSG registry and registry-driven operation selection.

use proj_core::{AreaOfInterest, Coord, SelectionOptions, Transform};

use crate::fixture::Fixture;
use crate::judge::Outcome;
use crate::util;

pub fn run(fx: &Fixture) -> Outcome {
    go(fx, false)
}

/// Same, with the source point as the area of interest (source CRS units):
/// what a caller has to add to get point-sensitive operation selection, which
/// PROJ does per coordinate on its own.
pub fn run_aoi(fx: &Fixture) -> Outcome {
    go(fx, true)
}

fn go(fx: &Fixture, aoi: bool) -> Outcome {
    let (src, dst) = (fx.src.as_str(), fx.dst());
    let mut o = Outcome { fed_src: Some(src.into()), fed_dst: Some(dst.into()), ..Default::default() };
    let s = match proj_wkt::parse_crs(src) {
        Ok(c) => c,
        Err(e) => {
            o.resolve_error = Some(format!("src: {e}"));
            return o;
        }
    };
    let d = match proj_wkt::parse_crs(dst) {
        Ok(c) => c,
        Err(e) => {
            o.resolve_error = Some(format!("dst: {e}"));
            return o;
        }
    };
    o.resolved = true;
    let made = if aoi {
        let p = fx.point();
        let opts = SelectionOptions::new().with_area_of_interest(AreaOfInterest::source_crs_point(Coord::new(p[0], p[1])));
        Transform::from_crs_defs_with_selection_options(&s, &d, opts)
    } else {
        Transform::from_crs_defs(&s, &d)
    };
    let t = match made {
        Ok(t) => t,
        Err(e) => {
            o.exec_error = Some(format!("transform: {e}"));
            return o;
        }
    };
    let op = t.selected_operation();
    o.operation = Some(format!("{} (accuracy {:?})", op.name, op.accuracy));
    let pt = util::src_point_deg(fx);
    if util::has_t(&pt) {
        o.notes.push("no epoch support: t dropped".into());
    }
    match conv(&t, &pt) {
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
        match t.inverse().map_err(|e| e.to_string()).and_then(|i| conv(&i, &f)) {
            Ok(v) => o.inv = Some(util::deg_to_src_native(fx, v)),
            Err(e) => o.inv_error = Some(e),
        }
    }
    o
}

fn conv(t: &Transform, p: &[f64]) -> Result<Vec<f64>, String> {
    if util::has_z(p) {
        let (x, y, z) = t.convert_3d((p[0], p[1], p[2])).map_err(|e| e.to_string())?;
        Ok(vec![x, y, z])
    } else {
        let (x, y) = t.convert((p[0], p[1])).map_err(|e| e.to_string())?;
        Ok(vec![x, y])
    }
}
