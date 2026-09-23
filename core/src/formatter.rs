pub fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    let mut size = bytes as f64;
    let mut idx = 0;

    // 1023.999 KiB moves up to "1.00 MiB" instead of printing "1024.00 KiB".
    while idx + 1 < UNITS.len() && (size * 100.0).round() >= 1024.0 * 100.0 {
        size /= 1024.0;
        idx += 1
    }

    if idx == 0 {
        format!("{} {}", bytes, UNITS[idx])
    } else {
        format!("{:.2} {}", size, UNITS[idx])
    }
}

#[cfg(test)]
mod tests {
    use super::format_size;

    #[test]
    fn format_bytes_under_1kb() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1), "1 B");
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn format_exact_kib() {
        assert_eq!(format_size(1024), "1.00 KiB");
        assert_eq!(format_size(1024 * 1024), "1.00 MiB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GiB");
        assert_eq!(format_size(1024_u64.pow(4)), "1.00 TiB");
    }

    #[test]
    fn format_partial_units() {
        assert_eq!(format_size(1536), "1.50 KiB");
        assert_eq!(format_size(5 * 1024 * 1024), "5.00 MiB");
        let bytes = 10 * 1024 * 1024 * 1024 + 512 * 1024 * 1024; // 10.5 GiB
        assert_eq!(format_size(bytes), "10.50 GiB");
    }

    #[test]
    fn format_rounds_up_to_next_unit() {
        assert_eq!(format_size(1024 * 1024 - 1), "1.00 MiB");
        assert_eq!(format_size(1024 * 1024 * 1024 - 1), "1.00 GiB");
        assert_eq!(format_size(1024_u64.pow(4) - 1), "1.00 TiB");
        assert_eq!(format_size(1024_u64.pow(5) - 1), "1.00 PiB");
        // Rounds to 1023.99, so it stays in KiB.
        assert_eq!(format_size(1_048_565), "1023.99 KiB");
    }

    #[test]
    fn format_large_values() {
        let one_pib = 1024_u64.pow(5);
        assert_eq!(format_size(one_pib), "1.00 PiB");

        let just_over_two_pib = one_pib * 2 + 100;
        assert_eq!(format_size(just_over_two_pib), "2.00 PiB");
    }
}
