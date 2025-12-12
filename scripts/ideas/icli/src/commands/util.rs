use termimad::{terminal_size, Area, MadSkin};

/// Truncate a string to a maximum length, adding an ellipsis if needed.
pub(crate) fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}

/// Strip leading "A " from description and capitalize first letter.
pub(crate) fn clean_desc(s: &str) -> String {
    let s = s.strip_prefix("A ").unwrap_or(s);
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Render markdown to terminal with colors and max width.
pub(crate) fn print_markdown(content: &str) {
    let skin = MadSkin::default();

    // Get terminal width, cap at 100 for readability
    let term_width = terminal_size().0.min(100);

    // Create an area with the desired width
    let area = Area::new(0, 0, term_width, 1000);

    // Get formatted text for the area width
    let text = skin.area_text(content, &area);
    print!("{}", text);
}

