//! QR code generation for terminal output.

use ansi_term::Color;
use qrcode::render::unicode::Dense1x2;
use qrcode::QrCode;

/// Render an ANSI-styled QR code for a URL, with optional line prefix indentation.
///
/// Uses Unicode half-block characters (`Dense1x2`) with explicit black-on-white
/// coloring so the QR code remains readable and scannable across both dark and
/// light terminal color schemes.
///
/// Returns `None` if the input cannot be encoded into a QR code.
pub fn render_qr_with_prefix(url: &str, prefix: &str) -> Option<String> {
    let code = QrCode::new(url.as_bytes()).ok()?;
    let image = code
        .render::<Dense1x2>()
        .dark_color(Dense1x2::Light)
        .light_color(Dense1x2::Dark)
        .quiet_zone(true)
        .build();

    let styled = image
        .lines()
        .map(|line| format!("{prefix}{}", Color::White.on(Color::Black).paint(line)))
        .collect::<Vec<_>>()
        .join("\n");

    Some(styled)
}

/// Render an ASCII/ANSI QR code for a given URL string indented by 2 spaces.
pub fn render_qr(url: &str) -> Option<String> {
    render_qr_with_prefix(url, "  ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_qr_standard_url() {
        let url = "https://sshx.io/s/abc1234567#key1234567890";
        let rendered = render_qr(url);
        assert!(rendered.is_some());
        let output = rendered.unwrap();
        assert!(!output.is_empty());
        // Check that lines have indentation and ANSI codes.
        for line in output.lines() {
            assert!(line.starts_with("  "));
            assert!(line.contains("\x1b[40;37m"));
        }
    }

    #[test]
    fn test_render_qr_with_prefix() {
        let url = "https://sshx.io/s/test#123";
        let prefix = ">>> ";
        let rendered = render_qr_with_prefix(url, prefix);
        assert!(rendered.is_some());
        let output = rendered.unwrap();
        for line in output.lines() {
            assert!(line.starts_with(prefix));
        }
    }

    #[test]
    fn test_render_qr_empty() {
        // Empty string can still be encoded as a valid minimal QR code.
        let rendered = render_qr("");
        assert!(rendered.is_some());
    }

    #[test]
    fn test_render_qr_read_write_urls() {
        let read_url = "https://sshx.io/s/abcdefghij#0123456789abcd";
        let write_url = "https://sshx.io/s/abcdefghij#0123456789abcd,9876543210zyxw";

        let qr_read = render_qr(read_url).expect("failed to render read-only qr");
        let qr_write = render_qr(write_url).expect("failed to render writable qr");

        assert!(!qr_read.is_empty());
        assert!(!qr_write.is_empty());

        // Check line counts: should be around 17-21 lines (compact for standard 80x24 terminal)
        let lines_read = qr_read.lines().count();
        let lines_write = qr_write.lines().count();
        assert!(lines_read <= 22, "QR code height too large: {lines_read}");
        assert!(lines_write <= 22, "QR code height too large: {lines_write}");
    }
}
