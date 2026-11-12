const BROWSER_PREFIXES: [&str; 2] = ["http://", "https://"];

pub fn launch(uri: &str) {
    if !is_browser_link(uri) {
        return;
    }

    let Ok(mut child) = std::process::Command::new("xdg-open").arg(uri).spawn() else {
        return;
    };

    std::thread::spawn(move || {
        let _ = child.wait();
    });
}

fn is_browser_link(uri: &str) -> bool {
    BROWSER_PREFIXES
        .iter()
        .any(|prefix| uri.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_links_are_recognized() {
        assert!(is_browser_link("https://example.com"));
        assert!(is_browser_link("http://example.com"));
        assert!(!is_browser_link("file:///tmp/document.md"));
        assert!(!is_browser_link("#anchor"));
    }
}
