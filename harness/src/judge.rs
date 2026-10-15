//! Judging: candidate outcome -> one JSON line, with differences from the
//! oracle in metres. Code here belongs to no candidate.

use serde::Serialize;
use serde_json::Value;

use crate::fixture::Fixture;

/// What a candidate adapter reports for one fixture.
#[derive(Debug, Default, Clone)]
pub struct Outcome {
    pub resolved: bool,
    pub resolve_error: Option<String>,
    pub executed: bool,
    pub exec_error: Option<String>,
    /// forward result: dst units, (east, north[, up[, t]]); geographic in degrees
    pub fwd: Option<Vec<f64>>,
    /// inverse of the oracle's forward result: src units; geographic in degrees
    pub inv: Option<Vec<f64>>,
    pub inv_error: Option<String>,
    /// the definitions actually handed to the library
    pub fed_src: Option<String>,
    pub fed_dst: Option<String>,
    /// operation the candidate reports it used, if it reports one
    pub operation: Option<String>,
    pub notes: Vec<String>,
    /// "n/a": the candidate is not meant to take this input (e.g. a lookup table and a WKT)
    pub not_applicable: bool,
}

impl Outcome {
    pub fn unresolved(e: impl ToString) -> Self {
        Outcome { resolve_error: Some(e.to_string()), ..Default::default() }
    }
    #[allow(dead_code)] // only the crs-definitions adapter uses it
    pub fn na(why: &str) -> Self {
        Outcome { not_applicable: true, resolve_error: Some(why.to_string()), ..Default::default() }
    }
    pub fn note(mut self, n: impl ToString) -> Self {
        self.notes.push(n.to_string());
        self
    }
}

#[derive(Serialize)]
pub struct Line<'a> {
    pub fixture: &'a str,
    pub group: &'a str,
    pub form: &'a str,
    pub candidate: &'a str,
    pub not_applicable: bool,
    pub resolved: bool,
    pub resolve_error: Option<String>,
    pub executed: bool,
    pub exec_error: Option<String>,
    pub point: Vec<f64>,
    pub fwd: Option<Vec<f64>>,
    pub oracle_fwd: Option<Vec<f64>>,
    pub oracle_executed: bool,
    pub oracle_error: Option<String>,
    pub oracle_operation: Option<String>,
    /// horizontal distance from the oracle forward result, metres
    pub diff_m: Option<f64>,
    /// same, with candidate axes swapped (flags an axis-order disagreement)
    pub diff_swapped_m: Option<f64>,
    pub dz_m: Option<f64>,
    pub inv: Option<Vec<f64>>,
    pub inv_error: Option<String>,
    /// distance of the candidate inverse from the oracle inverse, metres
    pub inv_diff_m: Option<f64>,
    /// distance from the forward result of another fixture (e.g. bare string vs authority code)
    pub compare_to: Option<String>,
    pub diff_vs_reference_m: Option<f64>,
    pub operation: Option<String>,
    pub fed_src: Option<String>,
    pub fed_dst: Option<String>,
    pub notes: Vec<String>,
    /// verdict against the oracle: exact (<1 mm) / close (<1 m) / off / failed / refused-like-oracle ...
    pub verdict: &'static str,
}

pub const M_PER_DEG: f64 = 111_320.0;

/// Horizontal distance in metres between two points in the same CRS.
/// geographic: degrees, longitude scaled by cos(lat), times 111320.
/// projected: native units times metres-per-unit. geocentric: 3D metres.
pub fn distance(fx: &Fixture, a: &[f64], b: &[f64], on_dst: bool) -> Option<f64> {
    if a.len() < 2 || b.len() < 2 {
        return None;
    }
    let (geog, geoc, unit) = if on_dst {
        (fx.dst_geographic(), fx.dst_geocentric(), fx.oracle.dst_unit_factor)
    } else {
        (fx.src_geographic(), fx.src_geocentric(), fx.oracle.src_unit_factor)
    };
    let d = if geog {
        let k = if on_dst { fx.dst_deg_per_unit() } else { fx.src_deg_per_unit() };
        let mut dlon = (a[0] - b[0]) * k;
        if dlon > 180.0 {
            dlon -= 360.0
        } else if dlon < -180.0 {
            dlon += 360.0
        }
        let dlat = (a[1] - b[1]) * k;
        let lat = (b[1] * k).to_radians();
        (dlon * lat.cos()).hypot(dlat) * M_PER_DEG
    } else if geoc {
        let dz = if a.len() > 2 && b.len() > 2 { a[2] - b[2] } else { 0.0 };
        ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + dz * dz).sqrt()
    } else {
        let f = match unit {
            Some(f) if f > 0.0 => f,
            _ => 1.0,
        };
        (a[0] - b[0]).hypot(a[1] - b[1]) * f
    };
    if d.is_finite() {
        Some(d)
    } else {
        None
    }
}

pub fn judge<'a>(fx: &'a Fixture, candidate: &'a str, o: Outcome, all: &[Fixture]) -> Line<'a> {
    let ofwd = fx.oracle_fwd();
    let oexec = fx.oracle.executed.unwrap_or(false);
    let fwd = o.fwd.clone().filter(|v| v.len() >= 2);
    let finite = fwd.as_ref().map(|v| v[0].is_finite() && v[1].is_finite()).unwrap_or(false);
    let diff_m = match (&fwd, &ofwd) {
        (Some(a), Some(b)) => distance(fx, a, b, true),
        _ => None,
    };
    let diff_swapped_m = match (&fwd, &ofwd) {
        (Some(a), Some(b)) => {
            let s = [a[1], a[0]];
            distance(fx, &s, b, true)
        }
        _ => None,
    };
    let dz_m = match (&fwd, &ofwd) {
        (Some(a), Some(b)) if a.len() > 2 && b.len() > 2 && !fx.dst_geocentric() => Some(a[2] - b[2]),
        _ => None,
    };
    let inv_diff_m = match (&o.inv, &fx.oracle.inv) {
        (Some(a), Some(b)) => distance(fx, a, b, false),
        _ => None,
    };
    let reference = fx
        .compare_to
        .as_ref()
        .and_then(|id| all.iter().find(|f| &f.id == id))
        .and_then(|f| f.oracle_fwd());
    let diff_vs_reference_m = match (&fwd, &reference) {
        (Some(a), Some(b)) => distance(fx, a, b, true),
        _ => None,
    };
    let oracle_operation = fx
        .oracle
        .op
        .as_ref()
        .and_then(|v| v.get("name"))
        .and_then(Value::as_str)
        .map(str::to_string);

    let executed = o.executed && finite;
    let verdict = if o.not_applicable {
        "n/a"
    } else if !oexec {
        // the oracle refused: agreement means refusing too
        if !o.resolved || !executed {
            "refused-like-oracle"
        } else {
            "answered-where-oracle-refused"
        }
    } else if !o.resolved {
        "unresolved"
    } else if !executed {
        "failed"
    } else {
        match diff_m {
            Some(d) if d < 0.001 => "exact",
            Some(d) if d < 1.0 => "close",
            Some(_) => match diff_swapped_m {
                Some(s) if s < 0.001 => "axis-swapped",
                _ => "off",
            },
            None => "failed",
        }
    };

    let mut notes = o.notes;
    if o.executed && !finite {
        notes.push("non-finite result reported as success".into());
    }
    Line {
        fixture: &fx.id,
        group: &fx.group,
        form: &fx.form,
        candidate,
        not_applicable: o.not_applicable,
        resolved: o.resolved,
        resolve_error: o.resolve_error,
        executed,
        exec_error: o.exec_error,
        point: fx.point(),
        fwd: o.fwd,
        oracle_fwd: ofwd,
        oracle_executed: oexec,
        oracle_error: fx.oracle.error.clone(),
        oracle_operation,
        diff_m,
        diff_swapped_m,
        dz_m,
        inv: o.inv,
        inv_error: o.inv_error,
        inv_diff_m,
        compare_to: fx.compare_to.clone(),
        diff_vs_reference_m,
        operation: o.operation,
        fed_src: o.fed_src,
        fed_dst: o.fed_dst,
        notes,
        verdict,
    }
}
