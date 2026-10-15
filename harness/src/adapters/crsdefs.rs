//! crs-definitions 0.6.0: the static EPSG -> proj4 table ports lean on,
//! executed by proj4rs. Only authority-code fixtures apply; the column
//! measures what the table's strings lose against PROJ's database.

use crate::fixture::{authority, Fixture};
use crate::judge::Outcome;
use crate::util;

fn lookup(def: &str) -> Result<String, String> {
    if util::is_crs84(def) {
        return Ok(util::HUB.to_string());
    }
    let Some((auth, code)) = authority(def) else {
        return Err("not an authority code".into());
    };
    if auth != "EPSG" {
        return Err(format!("{auth} codes not in table (EPSG only)"));
    }
    let n: u16 = code.parse().map_err(|_| format!("code {code} does not fit the u16 table key"))?;
    match crs_definitions::from_code(n) {
        Some(d) if !d.proj4.trim().is_empty() => Ok(d.proj4.to_string()),
        Some(_) => Err(format!("EPSG:{n} has an empty proj4 string")),
        None => Err(format!("EPSG:{n} not in table")),
    }
}

pub fn run(fx: &Fixture) -> Outcome {
    if authority(fx.dst()).is_none() && !util::is_crs84(fx.dst()) {
        return Outcome::na("input is not an authority code");
    }
    let src = match lookup(&fx.src) {
        Ok(s) => s,
        Err(e) => return Outcome::unresolved(format!("src: {e}")),
    };
    let dst = match lookup(fx.dst()) {
        Ok(s) => s,
        Err(e) => return Outcome::unresolved(format!("dst: {e}")),
    };
    super::proj4rs_ad::execute(fx, &src, &dst)
}
