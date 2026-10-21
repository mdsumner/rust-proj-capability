//! One module per candidate, each behind a cargo feature. An adapter turns a
//! fixture into an Outcome; it never judges.

use crate::fixture::Fixture;
use crate::judge::Outcome;

#[cfg(feature = "c-proj")]
mod proj_safe;
#[cfg(feature = "c-proj")]
mod proj_ffi;
#[cfg(feature = "c-proj4rs")]
mod proj4rs_ad;
#[cfg(feature = "c-crsdefs")]
mod crsdefs;
#[cfg(feature = "c-geodesy")]
mod geodesy_ad;
#[cfg(feature = "c-miniproj")]
mod miniproj_ad;
#[cfg(feature = "c-projcore")]
mod projcore;
#[cfg(feature = "c-wbprojection")]
mod wbprojection_ad;

pub struct Candidate {
    pub name: &'static str,
    pub run: Option<fn(&Fixture) -> Outcome>,
}

type RunFn = fn(&Fixture) -> Outcome;

macro_rules! cand {
    ($konst:ident, $feat:literal, $f:path) => {
        #[cfg(feature = $feat)]
        const $konst: Option<RunFn> = Some($f as RunFn);
        #[cfg(not(feature = $feat))]
        const $konst: Option<RunFn> = None;
    };
}

cand!(PROJ, "c-proj", proj_safe::run);
cand!(PROJ_SYS, "c-proj", proj_ffi::run);
cand!(PROJ4RS, "c-proj4rs", proj4rs_ad::run);
cand!(PROJ4RS_BOUND, "c-proj4rs", proj4rs_ad::run_bound);
cand!(CRSDEFS, "c-crsdefs", crsdefs::run);
cand!(GEODESY, "c-geodesy", geodesy_ad::run);
cand!(GEODESY_BOUND, "c-geodesy", geodesy_ad::run_bound);
cand!(MINIPROJ, "c-miniproj", miniproj_ad::run);
cand!(PROJCORE, "c-projcore", projcore::run);
cand!(PROJCORE_AOI, "c-projcore", projcore::run_aoi);
cand!(WBPROJECTION, "c-wbprojection", wbprojection_ad::run);

pub static REGISTRY: &[Candidate] = &[
    Candidate { name: "proj", run: PROJ },
    Candidate { name: "proj-sys", run: PROJ_SYS },
    Candidate { name: "proj4rs", run: PROJ4RS },
    Candidate { name: "proj4rs+bound", run: PROJ4RS_BOUND },
    Candidate { name: "crs-definitions", run: CRSDEFS },
    Candidate { name: "geodesy", run: GEODESY },
    Candidate { name: "geodesy+bound", run: GEODESY_BOUND },
    Candidate { name: "miniproj", run: MINIPROJ },
    Candidate { name: "proj-core", run: PROJCORE },
    Candidate { name: "proj-core+aoi", run: PROJCORE_AOI },
    Candidate { name: "wbprojection", run: WBPROJECTION },
];
