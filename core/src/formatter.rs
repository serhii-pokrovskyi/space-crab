pub struct SizeFormatter;

impl SizeFormatter {
    pub fn format(bytes: u64) -> String {
        const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
        let mut size = bytes as f64;
        let mut idx = 0;
        
        while idx + 1 < UNITS.len() && size >= 1024.0 {
            size /= 1024.0;
            idx += 1
        }
        
        if idx == 0 {
            format!("{} {}", bytes, UNITS[idx])
        } else {
            format!("{:.2} {}", size, UNITS[idx])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SizeFormatter;

    #[test]
    fn format_bytes_under_1kb() {
        assert_eq!(SizeFormatter::format(0), "0 B");
        assert_eq!(SizeFormatter::format(1), "1 B");
        assert_eq!(SizeFormatter::format(500), "500 B");
        assert_eq!(SizeFormatter::format(1023), "1023 B");
    }

    #[test]
    fn format_exact_kib() {
        assert_eq!(SizeFormatter::format(1024), "1.00 KiB");
        assert_eq!(SizeFormatter::format(1024 * 1024), "1.00 MiB");
        assert_eq!(SizeFormatter::format(1024 * 1024 * 1024), "1.00 GiB");
        assert_eq!(SizeFormatter::format(1024_u64.pow(4)), "1.00 TiB");
    }

    #[test]
    fn format_partial_units() {
        assert_eq!(SizeFormatter::format(1536), "1.50 KiB");
        assert_eq!(SizeFormatter::format(5 * 1024 * 1024), "5.00 MiB");
        let bytes = 10 * 1024 * 1024 * 1024 + 512 * 1024 * 1024; // 10.5 GiB
        assert_eq!(SizeFormatter::format(bytes), "10.50 GiB");
    }

    #[test]
    fn format_large_values() {
        let one_pib = 1024_u64.pow(5);
        assert_eq!(SizeFormatter::format(one_pib), "1.00 PiB");

        let almost_two_pib = one_pib * 2 + 100;
        assert_eq!(SizeFormatter::format(almost_two_pib), "2.00 PiB");
    }
}