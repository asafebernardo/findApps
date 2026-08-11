/// Formata bytes em string legível (pt-BR).
pub fn format_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let b = bytes as f64;
    if b >= GB {
        format!("{:.1} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_megabytes() {
        assert_eq!(format_size(245 * 1024 * 1024), "245.0 MB");
    }

    #[test]
    fn formats_gigabytes() {
        assert_eq!(format_size(12_400_000_000), "11.5 GB");
    }
}
