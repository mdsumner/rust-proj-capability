//! Fixture file (fixtures/crs.json, written by oracle/oracle.py) and the
//! oracle-side facts the harness judges against. Nothing here belongs to a
//! candidate. Some fields are read only by the docs generator
//! (report/make_docs.py), hence the allow.

#![allow(dead_code)]

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct FixtureFile {
    pub environment: Value,
    pub fixtures: Vec<Fixture>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Fixture {
    pub id: String,
    pub group: String,
    /// literal CRS definitions as the candidate receives them
    pub src: String,
    pub dst: Option<String>,
    pub src_ref: String,
    pub dst_ref: String,
    /// "authority" | "proj" | "wkt1-gdal" | "wkt1-esri" | "wkt2" | "projjson"
    pub form: String,
    /// source point, src units, visualisation (east, north) order; optional z, t
    pub point: Option<Vec<f64>>,
    #[serde(default)]
    pub compare_to: Option<String>,
    pub oracle: Oracle,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Exports {
    pub proj: Option<String>,
    pub proj_bound: Option<String>,
    pub wkt1_gdal_bound: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Oracle {
    #[serde(default)]
    pub resolved: Option<bool>,
    #[serde(default)]
    pub executed: Option<bool>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub src_kind: Option<String>,
    #[serde(default)]
    pub dst_kind: Option<String>,
    #[serde(default)]
    pub src_unit_factor: Option<f64>,
    #[serde(default)]
    pub dst_unit_factor: Option<f64>,
    #[serde(default)]
    pub src_export: Option<Exports>,
    #[serde(default)]
    pub dst_export: Option<Exports>,
    #[serde(default)]
    pub fwd: Option<Vec<Value>>,
    #[serde(default)]
    pub inv: Option<Vec<f64>>,
    #[serde(default)]
    pub op: Option<Value>,
}

impl Fixture {
    pub fn dst(&self) -> &str {
        self.dst.as_deref().unwrap_or("")
    }
    pub fn point(&self) -> Vec<f64> {
        self.point.clone().unwrap_or_default()
    }
    pub fn src_geographic(&self) -> bool {
        kind_is_geographic(self.oracle.src_kind.as_deref(), &self.src)
    }
    pub fn dst_geographic(&self) -> bool {
        kind_is_geographic(self.oracle.dst_kind.as_deref(), self.dst())
    }
    pub fn dst_geocentric(&self) -> bool {
        self.oracle.dst_kind.as_deref() == Some("geocentric")
    }
    pub fn src_geocentric(&self) -> bool {
        self.oracle.src_kind.as_deref() == Some("geocentric")
    }
    /// Degrees per native unit of a geographic CRS (1 for degrees, 0.9 for grads).
    pub fn src_deg_per_unit(&self) -> f64 {
        deg_per_unit(self.oracle.src_unit_factor)
    }
    pub fn dst_deg_per_unit(&self) -> f64 {
        deg_per_unit(self.oracle.dst_unit_factor)
    }
    /// Bound-to-WGS84 PROJ string PROJ exported for src (horizontal member).
    pub fn src_bound(&self) -> Option<&str> {
        self.oracle.src_export.as_ref().and_then(|e| e.proj_bound.as_deref())
    }
    pub fn dst_bound(&self) -> Option<&str> {
        self.oracle.dst_export.as_ref().and_then(|e| e.proj_bound.as_deref())
    }
    pub fn oracle_fwd(&self) -> Option<Vec<f64>> {
        let v = self.oracle.fwd.as_ref()?;
        let out: Vec<f64> = v.iter().filter_map(|x| x.as_f64()).collect();
        if out.len() == v.len() {
            Some(out)
        } else {
            None
        }
    }
}

fn kind_is_geographic(kind: Option<&str>, def: &str) -> bool {
    match kind {
        Some(k) => k == "geographic" || k == "compound-geographic",
        None => def.contains("proj=longlat") || def.contains("proj=latlong"),
    }
}

fn deg_per_unit(radians_per_unit: Option<f64>) -> f64 {
    match radians_per_unit {
        Some(f) if f > 0.0 && f < 1.0 => f.to_degrees(),
        _ => 1.0,
    }
}

/// Split "AUTH:CODE" (also "urn:ogc:def:crs:AUTH::CODE"-free plain forms).
pub fn authority(def: &str) -> Option<(String, String)> {
    let d = def.trim();
    if d.starts_with('+') || d.starts_with('{') || d.contains('[') {
        return None;
    }
    let (a, c) = d.split_once(':')?;
    Some((a.to_ascii_uppercase(), c.to_string()))
}

pub fn is_proj_string(def: &str) -> bool {
    def.trim_start().starts_with('+') || def.trim_start().starts_with("proj=")
}

pub fn is_wkt(def: &str) -> bool {
    let d = def.trim_start();
    !d.starts_with('{') && d.contains('[')
}

pub fn is_projjson(def: &str) -> bool {
    def.trim_start().starts_with('{')
}
