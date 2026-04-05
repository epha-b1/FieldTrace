/// Format: FAC01-YYYYMMDD-SEQ4-CHECK
/// CHECK is a mod-10 Luhn-style checksum over the digits.

pub fn generate(facility_code: &str, date: &str, seq: u32) -> String {
    let base = format!("{}-{}-{:04}", facility_code, date, seq);
    let checksum = luhn_checksum(&base);
    format!("{}-{}", base, checksum)
}

pub fn verify(code: &str) -> bool {
    let parts: Vec<&str> = code.rsplitn(2, '-').collect();
    if parts.len() != 2 { return false; }
    let check: u32 = match parts[0].parse() { Ok(v) => v, Err(_) => return false };
    let base = parts[1];
    luhn_checksum(base) == check
}

fn luhn_checksum(input: &str) -> u32 {
    let digits: Vec<u32> = input.chars().filter_map(|c| c.to_digit(10)).collect();
    let sum: u32 = digits.iter().rev().enumerate().map(|(i, d)| {
        if i % 2 == 0 {
            let doubled = d * 2;
            if doubled > 9 { doubled - 9 } else { doubled }
        } else { *d }
    }).sum();
    (10 - (sum % 10)) % 10
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let c = generate("FAC01", "20260404", 42);
        assert!(verify(&c));
        let tampered = c.replacen("42", "43", 1);
        assert!(!verify(&tampered));
    }

    #[test]
    fn format_matches_spec() {
        // FAC01-YYYYMMDD-SEQ4-CHECK
        let c = generate("FAC01", "20260405", 1);
        assert!(c.starts_with("FAC01-20260405-0001-"));
        // last segment is a single digit (0..=9)
        let last = c.rsplit('-').next().unwrap();
        assert_eq!(last.len(), 1);
        assert!(last.chars().all(|d| d.is_ascii_digit()));
    }

    #[test]
    fn different_dates_produce_different_checksums_when_digits_differ() {
        let a = generate("FAC01", "20260101", 1);
        let b = generate("FAC01", "20260201", 1);
        assert_ne!(a, b);
        assert!(verify(&a));
        assert!(verify(&b));
    }

    #[test]
    fn malformed_verify_returns_false() {
        assert!(!verify(""));
        assert!(!verify("no-checksum-part"));
        assert!(!verify("FAC01-20260404-0042"));
    }

    #[test]
    fn sequence_padding_is_four_digits() {
        let c = generate("FAC01", "20260405", 7);
        assert!(c.contains("-0007-"));
        let c2 = generate("FAC01", "20260405", 1234);
        assert!(c2.contains("-1234-"));
    }
}
