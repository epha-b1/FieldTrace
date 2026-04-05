// Deterministic parsing rules for supply entries
use std::collections::HashMap;

pub fn normalize_color(raw: &str) -> Option<String> {
    let map: HashMap<&str, &str> = [
        ("navy", "blue"), ("royal", "blue"), ("sky", "blue"), ("blue", "blue"),
        ("crimson", "red"), ("scarlet", "red"), ("red", "red"),
        ("emerald", "green"), ("forest", "green"), ("lime", "green"), ("green", "green"),
        ("black", "black"), ("white", "white"), ("gray", "gray"), ("grey", "gray"),
        ("yellow", "yellow"), ("gold", "yellow"),
        ("brown", "brown"), ("tan", "brown"),
        ("purple", "purple"), ("violet", "purple"),
        ("orange", "orange"),
        ("pink", "pink"),
    ].iter().cloned().collect();
    map.get(raw.trim().to_lowercase().as_str()).map(|s| s.to_string())
}

/// Convert size with unit to canonical form.
/// Weights → lb (1 lb = 16 oz). Lengths → ft (1 ft = 12 in).
pub fn normalize_size(raw: &str) -> Option<String> {
    let s = raw.trim().to_lowercase();
    // Parse "NN unit"
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 2 { return None; }
    let n: f64 = parts[0].parse().ok()?;
    let unit = parts[1];
    match unit {
        "oz" => Some(format!("{:.4} lb", n / 16.0)),
        "lb" | "lbs" => Some(format!("{:.4} lb", n)),
        "in" | "inch" | "inches" => Some(format!("{:.4} ft", n / 12.0)),
        "ft" | "feet" => Some(format!("{:.4} ft", n)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_navy_maps_to_blue() {
        assert_eq!(normalize_color("navy"), Some("blue".into()));
        assert_eq!(normalize_color(" Navy "), Some("blue".into()));
        assert_eq!(normalize_color("ROYAL"), Some("blue".into()));
    }

    #[test]
    fn color_unknown_returns_none() {
        assert_eq!(normalize_color("teal"), None);
        assert_eq!(normalize_color(""), None);
    }

    #[test]
    fn color_gray_and_grey_both_work() {
        assert_eq!(normalize_color("gray"), Some("gray".into()));
        assert_eq!(normalize_color("grey"), Some("gray".into()));
    }

    #[test]
    fn size_weight_conversion() {
        assert_eq!(normalize_size("16 oz"), Some("1.0000 lb".into()));
        assert_eq!(normalize_size("1 lb"), Some("1.0000 lb".into()));
        assert_eq!(normalize_size("2 lbs"), Some("2.0000 lb".into()));
    }

    #[test]
    fn size_length_conversion() {
        assert_eq!(normalize_size("12 in"), Some("1.0000 ft".into()));
        assert_eq!(normalize_size("24 inches"), Some("2.0000 ft".into()));
        assert_eq!(normalize_size("3 ft"), Some("3.0000 ft".into()));
    }

    #[test]
    fn size_malformed_returns_none() {
        assert_eq!(normalize_size(""), None);
        assert_eq!(normalize_size("heavy"), None);
        assert_eq!(normalize_size("12"), None);
        assert_eq!(normalize_size("12 lightyears"), None);
    }
}
