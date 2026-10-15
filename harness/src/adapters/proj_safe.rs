//! georust `proj` 0.31, safe API only: Proj::new to resolve each CRS,
//! Proj::new_known_crs (crs_to_crs + normalize_for_visualization) to execute.
//! convert() is 2D: z and t are not passed through the safe API.

use proj::Proj;

use crate::fixture::Fixture;
use crate::judge::Outcome;
use crate::util;

pub fn run(fx: &Fixture) -> Outcome {
    let ((src, a), (dst, b)) = (util::crs_typed(&fx.src), util::crs_typed(fx.dst()));
    let (src, dst) = (src.as_str(), dst.as_str());
    let mut o = Outcome { fed_src: Some(src.into()), fed_dst: Some(dst.into()), ..Default::default() };
    if a || b {
        o.notes.push("appended +type=crs (crs_to_crs rejects a bare PROJ operation string)".into());
    }
    for d in [src, dst] {
        if let Err(e) = Proj::new(d) {
            o.resolve_error = Some(format!("{d:.60}: {e}"));
            return o;
        }
    }
    o.resolved = true;
    let p = match Proj::new_known_crs(src, dst, None) {
        Ok(p) => p,
        Err(e) => {
            o.exec_error = Some(format!("new_known_crs: {e}"));
            return o;
        }
    };
    let info = p.proj_info();
    o.operation = info.description.clone();
    let pt = fx.point();
    if util::has_z(&pt) {
        o.notes.push("safe convert() is 2D: z and t dropped".into());
    }
    match p.convert((pt[0], pt[1])) {
        Ok((x, y)) => {
            o.executed = true;
            o.fwd = Some(vec![x, y]);
        }
        Err(e) => {
            o.exec_error = Some(e.to_string());
            return o;
        }
    }
    if let Some(f) = fx.oracle_fwd() {
        match Proj::new_known_crs(dst, src, None).map_err(|e| e.to_string()).and_then(|q| {
            q.convert((f[0], f[1])).map_err(|e| e.to_string())
        }) {
            Ok((x, y)) => o.inv = Some(vec![x, y]),
            Err(e) => o.inv_error = Some(e),
        }
    }
    o
}
