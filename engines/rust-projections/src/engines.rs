//! One adapter per engine. An adapter prepares a lon/lat <-> projected
//! transform from the bare PROJ string and the matching lon/lat definition
//! (same earth model, so no datum step); it never judges.

use crate::{Engine, Prepared, Projection};

pub static REGISTRY: &[Engine] = &[
    Engine { name: "proj4rs", version: "0.2.1", prepare: proj4rs_release::prepare },
    Engine { name: "proj4rs-main", version: "git main", prepare: proj4rs_git::prepare },
    Engine { name: "proj-core", version: "0.12.0", prepare: projcore::prepare },
    Engine { name: "wbprojection", version: "0.3.3", prepare: wb::prepare },
    Engine { name: "geodesy", version: "0.15.0", prepare: geodesy_ad::prepare },
    Engine { name: "terrio", version: "0.5.0", prepare: terrio_ad::prepare },
];

/// proj4rs works in radians on the geographic side.
macro_rules! proj4rs_adapter {
    ($modname:ident, $krate:ident) => {
        mod $modname {
            use super::*;
            use ::$krate::transform::transform;
            use ::$krate::Proj;

            struct T {
                geo: Proj,
                prj: Proj,
            }

            pub fn prepare(p: &Projection) -> Result<Box<dyn Prepared>, String> {
                let geo = Proj::from_proj_string(&p.geographic).map_err(|e| format!("geographic: {e}"))?;
                let prj = Proj::from_proj_string(&p.proj).map_err(|e| e.to_string())?;
                Ok(Box::new(T { geo, prj }))
            }

            impl Prepared for T {
                fn fwd(&self, lon: f64, lat: f64) -> Option<[f64; 2]> {
                    let mut c = (lon.to_radians(), lat.to_radians(), 0.0);
                    transform(&self.geo, &self.prj, &mut c).ok()?;
                    Some([c.0, c.1])
                }
                fn inv(&self, x: f64, y: f64) -> Option<[f64; 2]> {
                    let mut c = (x, y, 0.0);
                    transform(&self.prj, &self.geo, &mut c).ok()?;
                    Some([c.0.to_degrees(), c.1.to_degrees()])
                }
            }
        }
    };
}

proj4rs_adapter!(proj4rs_release, proj4rs);
proj4rs_adapter!(proj4rs_git, proj4rs_main);

mod projcore {
    use super::*;
    use proj_core::Transform;

    struct T {
        f: Transform,
        i: Transform,
    }

    pub fn prepare(p: &Projection) -> Result<Box<dyn Prepared>, String> {
        let s = proj_wkt::parse_crs(&p.geographic).map_err(|e| format!("geographic: {e}"))?;
        let d = proj_wkt::parse_crs(&p.proj).map_err(|e| e.to_string())?;
        let f = Transform::from_crs_defs(&s, &d).map_err(|e| e.to_string())?;
        let i = f.inverse().map_err(|e| e.to_string())?;
        Ok(Box::new(T { f, i }))
    }

    impl Prepared for T {
        fn fwd(&self, lon: f64, lat: f64) -> Option<[f64; 2]> {
            self.f.convert((lon, lat)).ok().map(|(x, y)| [x, y])
        }
        fn inv(&self, x: f64, y: f64) -> Option<[f64; 2]> {
            self.i.convert((x, y)).ok().map(|(x, y)| [x, y])
        }
    }
}

mod wb {
    use super::*;
    use wbprojection::Crs;

    struct T {
        geo: Crs,
        prj: Crs,
    }

    pub fn prepare(p: &Projection) -> Result<Box<dyn Prepared>, String> {
        let geo = wbprojection::from_proj_string(&p.geographic).map_err(|e| format!("geographic: {e}"))?;
        let prj = wbprojection::from_proj_string(&p.proj).map_err(|e| e.to_string())?;
        Ok(Box::new(T { geo, prj }))
    }

    impl Prepared for T {
        fn fwd(&self, lon: f64, lat: f64) -> Option<[f64; 2]> {
            self.geo.transform_to(lon, lat, &self.prj).ok().map(|(x, y)| [x, y])
        }
        fn inv(&self, x: f64, y: f64) -> Option<[f64; 2]> {
            self.prj.transform_to(x, y, &self.geo).ok().map(|(x, y)| [x, y])
        }
    }
}

mod geodesy_ad {
    use super::*;
    use geodesy::authoring::parse_proj;
    use geodesy::prelude::*;

    struct T {
        ctx: Minimal,
        op: OpHandle,
    }

    /// geodesy's own PROJ-string translation, run as a single operator.
    pub fn prepare(p: &Projection) -> Result<Box<dyn Prepared>, String> {
        let def = parse_proj(&p.proj).map_err(|e| format!("parse_proj: {e}"))?;
        let mut ctx = Minimal::new();
        let op = ctx.op(def.trim()).map_err(|e| e.to_string())?;
        Ok(Box::new(T { ctx, op }))
    }

    impl T {
        fn go(&self, dir: Direction, a: f64, b: f64) -> Option<[f64; 2]> {
            let mut c = [Coor2D::raw(a, b)];
            self.ctx.apply(self.op, dir, &mut c).ok()?;
            Some([c[0][0], c[0][1]])
        }
    }

    impl Prepared for T {
        fn fwd(&self, lon: f64, lat: f64) -> Option<[f64; 2]> {
            self.go(Fwd, lon.to_radians(), lat.to_radians())
        }
        fn inv(&self, x: f64, y: f64) -> Option<[f64; 2]> {
            self.go(Inv, x, y).map(|v| [v[0].to_degrees(), v[1].to_degrees()])
        }
    }
}

/// terrio's CRS engine: its own ports of PROJ's methods, wbprojection as the
/// fallback. Two transformers, since a Transformer runs one way.
mod terrio_ad {
    use super::*;
    use terrio::crs::{SpatialRef, Transformer};

    struct T {
        fwd: Transformer,
        inv: Transformer,
    }

    pub fn prepare(p: &Projection) -> Result<Box<dyn Prepared>, String> {
        let geo = SpatialRef::from_proj4(&p.geographic).map_err(|e| format!("geographic: {e}"))?;
        let prj = SpatialRef::from_proj4(&p.proj).map_err(|e| e.to_string())?;
        let fwd = Transformer::new(&geo, &prj).map_err(|e| e.to_string())?;
        let inv = Transformer::new(&prj, &geo).map_err(|e| e.to_string())?;
        Ok(Box::new(T { fwd, inv }))
    }

    impl Prepared for T {
        fn fwd(&self, lon: f64, lat: f64) -> Option<[f64; 2]> {
            self.fwd.transform(lon, lat).ok().map(|(x, y)| [x, y])
        }
        fn inv(&self, x: f64, y: f64) -> Option<[f64; 2]> {
            self.inv.transform(x, y).ok().map(|(x, y)| [x, y])
        }
    }
}
