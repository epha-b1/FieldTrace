//! Cross-cutting utilities: error sanitization, date formatting, role helpers.
//!
//! ## Error sanitization
//!
//! DB and system errors must NEVER propagate their raw `Display` output to
//! clients. This module wraps the common "map_err → AppError::internal"
//! pattern into `db_err(trace_id)` / `system_err(trace_id)` helpers that log
//! full detail via `tracing::error!` and return a generic user-facing message.
//!
//! ## Local date/time formatting
//!
//! Evidence watermark and traceability codes need real local dates without a
//! chrono dependency. We compute civil date/time fields directly from
//! `SystemTime` using the standard proleptic Gregorian calendar.

use crate::error::AppError;
use crate::extractors::SessionUser;
use std::time::{SystemTime, UNIX_EPOCH};

// ── Error sanitization ────────────────────────────────────────────────

/// Returns a mapping closure that logs the full DB error and yields a
/// sanitized AppError::internal response (no raw `Display` leaked).
pub fn db_err<E: std::fmt::Display>(trace_id: &str) -> impl Fn(E) -> AppError + '_ {
    let tid = trace_id.to_string();
    move |e| {
        tracing::error!(trace_id = %tid, error = %e, "Database error");
        AppError::internal("Internal server error", tid.clone())
    }
}

/// System-level (I/O, crypto, other) error sanitizer.
pub fn system_err<E: std::fmt::Display>(trace_id: &str, context: &'static str) -> impl Fn(E) -> AppError + '_ {
    let tid = trace_id.to_string();
    move |e| {
        tracing::error!(trace_id = %tid, context = %context, error = %e, "System error");
        AppError::internal("Internal server error", tid.clone())
    }
}

// ── Role authorization helpers ────────────────────────────────────────

pub fn is_admin(u: &SessionUser) -> bool { u.role == "administrator" }
pub fn is_staff(u: &SessionUser) -> bool { u.role == "operations_staff" }
pub fn is_auditor(u: &SessionUser) -> bool { u.role == "auditor" }

/// Reject auditors from mutating endpoints. Returns 403 if role == auditor.
pub fn require_write_role(u: &SessionUser, trace_id: &str) -> Result<(), AppError> {
    if is_auditor(u) {
        return Err(AppError::forbidden(
            "Auditors have read-only access and cannot perform this action",
            trace_id,
        ));
    }
    Ok(())
}

/// Admin or auditor only (used for traceability publish/retract, exports).
pub fn require_admin_or_auditor(u: &SessionUser, trace_id: &str) -> Result<(), AppError> {
    if !is_admin(u) && !is_auditor(u) {
        return Err(AppError::forbidden("Administrator or Auditor role required", trace_id));
    }
    Ok(())
}

// ── Local date/time formatting (no chrono) ────────────────────────────

/// Civil date computed from a Unix timestamp (UTC — acceptable for an
/// offline facility app where local = facility time).
#[derive(Debug, Clone, Copy)]
pub struct CivilDateTime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl CivilDateTime {
    pub fn from_unix(ts: i64) -> Self {
        // Days since Unix epoch (1970-01-01).
        let secs_per_day = 86_400i64;
        let mut days = ts.div_euclid(secs_per_day);
        let mut secs_of_day = ts.rem_euclid(secs_per_day) as i32;
        if secs_of_day < 0 { secs_of_day += secs_per_day as i32; days -= 1; }

        // Civil from days algorithm by Howard Hinnant.
        let z = days + 719_468; // shift epoch to 0000-03-01
        let era = if z >= 0 { z } else { z - 146096 } / 146097;
        let doe = (z - era * 146097) as u32; // [0, 146096]
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
        let y = yoe as i32 + era as i32 * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
        let mp = (5 * doy + 2) / 153; // [0, 11]
        let d = (doy - (153 * mp + 2) / 5 + 1) as u8; // [1, 31]
        let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u8; // [1, 12]
        let year = if m <= 2 { y + 1 } else { y };

        let hour = (secs_of_day / 3600) as u8;
        let minute = ((secs_of_day % 3600) / 60) as u8;
        let second = (secs_of_day % 60) as u8;

        CivilDateTime { year, month: m, day: d, hour, minute, second }
    }

    pub fn now() -> Self {
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
        Self::from_unix(ts)
    }

    /// `YYYYMMDD`
    pub fn yyyymmdd(&self) -> String {
        format!("{:04}{:02}{:02}", self.year, self.month, self.day)
    }

    /// `MM/DD/YYYY hh:mm AM/PM` (12-hour clock)
    pub fn us_12h(&self) -> String {
        let (h12, ampm) = match self.hour {
            0 => (12u8, "AM"),
            h if h < 12 => (h, "AM"),
            12 => (12, "PM"),
            h => (h - 12, "PM"),
        };
        format!("{:02}/{:02}/{:04} {:02}:{:02} {}",
            self.month, self.day, self.year, h12, self.minute, ampm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_epoch_is_1970_01_01() {
        let d = CivilDateTime::from_unix(0);
        assert_eq!((d.year, d.month, d.day), (1970, 1, 1));
        assert_eq!((d.hour, d.minute, d.second), (0, 0, 0));
    }

    #[test]
    fn mid_2020_date() {
        // 2020-07-04 00:00:00 UTC = 1593820800
        let d = CivilDateTime::from_unix(1_593_820_800);
        assert_eq!((d.year, d.month, d.day), (2020, 7, 4));
    }

    #[test]
    fn yyyymmdd_format() {
        let d = CivilDateTime { year: 2026, month: 4, day: 5, hour: 0, minute: 0, second: 0 };
        assert_eq!(d.yyyymmdd(), "20260405");
    }

    #[test]
    fn us_12h_morning() {
        let d = CivilDateTime { year: 2026, month: 4, day: 5, hour: 9, minute: 30, second: 0 };
        assert_eq!(d.us_12h(), "04/05/2026 09:30 AM");
    }

    #[test]
    fn us_12h_noon() {
        let d = CivilDateTime { year: 2026, month: 4, day: 5, hour: 12, minute: 0, second: 0 };
        assert_eq!(d.us_12h(), "04/05/2026 12:00 PM");
    }

    #[test]
    fn us_12h_midnight() {
        let d = CivilDateTime { year: 2026, month: 4, day: 5, hour: 0, minute: 0, second: 0 };
        assert_eq!(d.us_12h(), "04/05/2026 12:00 AM");
    }

    #[test]
    fn us_12h_evening() {
        let d = CivilDateTime { year: 2026, month: 4, day: 5, hour: 15, minute: 45, second: 0 };
        assert_eq!(d.us_12h(), "04/05/2026 03:45 PM");
    }
}
