//! `proj-sys` 0.27 raw FFI against the same libproj: what the safe API does
//! not expose. 4D proj_trans (z and epoch), proj_trans_get_last_used_operation,
//! and proj_crs_create_bound_crs_to_WGS84 + proj_as_proj_string (the export a
//! wrapper would hand to a port).

use std::ffi::{CStr, CString};
use std::ptr::null_mut;

use proj_sys::*;

use crate::fixture::Fixture;
use crate::judge::Outcome;

struct Ctx(*mut PJ_CONTEXT);
impl Drop for Ctx {
    fn drop(&mut self) {
        unsafe {
            proj_context_destroy(self.0);
        }
    }
}
struct Pj(*mut PJ);
impl Drop for Pj {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                proj_destroy(self.0);
            }
        }
    }
}

unsafe fn errstr(ctx: *mut PJ_CONTEXT) -> String {
    let e = proj_context_errno(ctx);
    let s = proj_context_errno_string(ctx, e);
    if s.is_null() {
        format!("errno {e}")
    } else {
        CStr::from_ptr(s).to_string_lossy().into_owned()
    }
}

unsafe fn name_of(pj: *mut PJ) -> Option<String> {
    if pj.is_null() {
        return None;
    }
    let n = proj_get_name(pj);
    (!n.is_null()).then(|| CStr::from_ptr(n).to_string_lossy().into_owned())
}

unsafe fn trans(ctx: *mut PJ_CONTEXT, pj: *mut PJ, dir: PJ_DIRECTION, p: &[f64]) -> Result<Vec<f64>, String> {
    let c = PJ_COORD {
        xyzt: PJ_XYZT {
            x: p[0],
            y: p[1],
            z: p.get(2).copied().unwrap_or(0.0),
            t: p.get(3).copied().unwrap_or(f64::INFINITY),
        },
    };
    proj_errno_reset(pj);
    let r = proj_trans(pj, dir, c);
    let e = proj_errno(pj);
    if e != 0 {
        let s = proj_context_errno_string(ctx, e);
        return Err(if s.is_null() { format!("errno {e}") } else { CStr::from_ptr(s).to_string_lossy().into_owned() });
    }
    let v = [r.xyzt.x, r.xyzt.y, r.xyzt.z, r.xyzt.t];
    Ok(v[..p.len().clamp(2, 4)].to_vec())
}

unsafe fn bound_proj_string(ctx: *mut PJ_CONTEXT, crs: *mut PJ) -> Option<String> {
    let b = Pj(proj_crs_create_bound_crs_to_WGS84(ctx, crs, null_mut()));
    if b.0.is_null() {
        return None;
    }
    let s = proj_as_proj_string(ctx, b.0, PJ_PROJ_STRING_TYPE_PJ_PROJ_5, null_mut());
    (!s.is_null()).then(|| CStr::from_ptr(s).to_string_lossy().into_owned())
}

pub fn run(fx: &Fixture) -> Outcome {
    unsafe { run_inner(fx) }
}

unsafe fn run_inner(fx: &Fixture) -> Outcome {
    let ((src, a), (dst, b)) = (crate::util::crs_typed(&fx.src), crate::util::crs_typed(fx.dst()));
    let (src, dst) = (src.as_str(), dst.as_str());
    let mut o = Outcome { fed_src: Some(src.into()), fed_dst: Some(dst.into()), ..Default::default() };
    if a || b {
        o.notes.push("appended +type=crs (crs_to_crs rejects a bare PROJ operation string)".into());
    }
    let ctx = Ctx(proj_context_create());
    proj_context_set_enable_network(ctx.0, 0);
    let (Ok(cs), Ok(cd)) = (CString::new(src), CString::new(dst)) else {
        return Outcome::unresolved("NUL in definition");
    };
    let s = Pj(proj_create(ctx.0, cs.as_ptr()));
    if s.0.is_null() {
        o.resolve_error = Some(format!("src: {}", errstr(ctx.0)));
        return o;
    }
    let d = Pj(proj_create(ctx.0, cd.as_ptr()));
    if d.0.is_null() {
        o.resolve_error = Some(format!("dst: {}", errstr(ctx.0)));
        return o;
    }
    o.resolved = true;
    if let Some(b) = bound_proj_string(ctx.0, d.0) {
        o.notes.push(format!("bound dst export: {b}"));
    }
    let raw = Pj(proj_create_crs_to_crs_from_pj(ctx.0, s.0, d.0, null_mut(), std::ptr::null()));
    if raw.0.is_null() {
        o.exec_error = Some(format!("crs_to_crs: {}", errstr(ctx.0)));
        return o;
    }
    let pj = Pj(proj_normalize_for_visualization(ctx.0, raw.0));
    if pj.0.is_null() {
        o.exec_error = Some(format!("normalize: {}", errstr(ctx.0)));
        return o;
    }
    let pt = fx.point();
    match trans(ctx.0, pj.0, PJ_DIRECTION_PJ_FWD, &pt) {
        Ok(v) => {
            o.executed = true;
            o.fwd = Some(v);
        }
        Err(e) => {
            o.exec_error = Some(e);
            return o;
        }
    }
    let last = Pj(proj_trans_get_last_used_operation(pj.0));
    o.operation = name_of(last.0).or_else(|| name_of(pj.0));
    if let Some(f) = fx.oracle_fwd() {
        match trans(ctx.0, pj.0, PJ_DIRECTION_PJ_INV, &f) {
            Ok(v) => o.inv = Some(v),
            Err(e) => o.inv_error = Some(e),
        }
    }
    o
}
