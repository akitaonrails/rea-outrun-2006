//! Section-aware INI editing for `OutRun2006Tweaks.ini`, which keeps every
//! comment and unmanaged key exactly as the user left it.
//!
//! `outrun2006.ini` has no sections; the plain `set_key` in `config` and
//! this module's own key matching both accept `key = value`,
//! `key=value` and indented lines.

/// Sets `key` inside `[section]`, replacing its line in place (keeping the
/// surrounding comments), inserting it at the section's end when missing,
/// and appending a new section at the end of the file when that is missing
/// too. Other lines are untouched.
pub fn set_section_key(text: &str, section: &str, key: &str, value: &str) -> String {
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let in_section = |header: &str| header.trim().eq_ignore_ascii_case(&format!("[{section}]"));
    let mut start = None;
    let mut end = lines.len();
    for (n, line) in lines.iter().enumerate() {
        let t = line.trim();
        if t.starts_with('[') && t.ends_with(']') {
            if start.is_some() {
                end = n;
                break;
            }
            if in_section(t) {
                start = Some(n);
            }
        }
    }
    let Some(start) = start else {
        // The section does not exist yet: add it, with the key.
        if !lines.is_empty() && !lines.last().is_some_and(|l| l.trim().is_empty()) {
            lines.push(String::new());
        }
        lines.push(format!("[{section}]"));
        lines.push(format!("{key} = {value}"));
        return join(lines, text);
    };
    let is_key = |line: &str| {
        let t = line.trim();
        !t.starts_with('#')
            && !t.starts_with('[')
            && t.split_once('=').is_some_and(|(k, _)| k.trim() == key)
    };
    for line in &mut lines[start + 1..end] {
        if is_key(line) {
            *line = format!("{key} = {value}");
            return join(lines, text);
        }
    }
    // Not present: insert at the section's end, before the blank line that
    // usually separates sections.
    let mut at = end;
    while at > start + 1 && lines[at - 1].trim().is_empty() {
        at -= 1;
    }
    lines.insert(at, format!("{key} = {value}"));
    join(lines, text)
}

/// Removes every active `key = …` line inside `[section]`.
pub fn remove_section_key(text: &str, section: &str, key: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') && t.ends_with(']') {
            inside = t.eq_ignore_ascii_case(&format!("[{section}]"));
            lines.push(line.to_string());
            continue;
        }
        let is_key = inside
            && !t.starts_with('#')
            && t.split_once('=').is_some_and(|(k, _)| k.trim() == key);
        if !is_key {
            lines.push(line.to_string());
        }
    }
    join(lines, text)
}

/// Every active `(section, key)` pair in the file.
pub fn section_keys(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut section = None;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') && t.ends_with(']') {
            section = Some(t[1..t.len() - 1].trim().to_string());
        } else if !t.starts_with('#')
            && let (Some(section), Some((key, _))) = (&section, t.split_once('='))
        {
            out.push((section.clone(), key.trim().to_string()));
        }
    }
    out
}

/// Reads `key` from `[section]`.
pub fn section_key(text: &str, section: &str, key: &str) -> Option<String> {
    let mut inside = false;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') && t.ends_with(']') {
            inside = t.eq_ignore_ascii_case(&format!("[{section}]"));
            continue;
        }
        if inside
            && !t.starts_with('#')
            && let Some((k, v)) = t.split_once('=')
            && k.trim() == key
        {
            return Some(v.trim().to_string());
        }
    }
    None
}

/// Rebuilds the file, keeping the original's trailing newline shape (a
/// file without one keeps its shape; an empty file gains one).
fn join(lines: Vec<String>, original: &str) -> String {
    let mut out = String::new();
    for line in lines {
        out.push_str(&line);
        out.push('\n');
    }
    if !original.is_empty() && !original.ends_with('\n') && out.ends_with('\n') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = "# OutRun2006Tweaks\n\n[Performance]\n# a comment\nFramerateLimit = 60\n\n[Controls]\nSteeringDeadZone = 0.2\nVibrationMode=0\n\n[Graphics]\nDrawDistanceIncrease = 0\n\n[CDTracks]\n01_Splash_wave.ogg = Splash Wave\nLast_Wave.ogg = Last Wave\nor2ed1.ogg = ending\n";

    #[test]
    fn replaces_keys_in_place_and_keeps_comments() {
        let out = set_section_key(FILE, "Performance", "FramerateLimit", "0");
        assert!(out.contains("# a comment\n"));
        assert!(out.contains("FramerateLimit = 0"));
        assert_eq!(out.lines().count(), FILE.lines().count());
        // The other sections are untouched.
        assert!(out.contains("VibrationMode=0"));
        // A key in another section is not touched.
        let out = set_section_key(FILE, "Graphics", "DrawDistanceIncrease", "4");
        assert!(out.contains("DrawDistanceIncrease = 4"));
        assert!(out.contains("FramerateLimit = 60"));
    }

    #[test]
    fn adds_missing_keys_inside_the_section() {
        let out = set_section_key(FILE, "Controls", "VibrationMode", "1");
        assert!(out.contains("VibrationMode = 1"));
        assert!(!out.contains("VibrationMode=0"));
        // VibrationMode replaced? No: VibrationMode=0 exists, so it is
        // replaced, not added. Use a genuinely missing key:
        let out = set_section_key(FILE, "Controls", "UseNewInput", "true");
        let controls = &out[out.find("[Controls]").unwrap()..out.find("[Graphics]").unwrap()];
        assert!(controls.contains("UseNewInput = true"));
        assert!(controls.contains("SteeringDeadZone = 0.2"));
        assert!(controls.trim_end().ends_with("UseNewInput = true"));
        // A key added to the last section lands inside it.
        let out = set_section_key(FILE, "CDTracks", "New_Track.ogg", "New Track");
        assert!(out.contains("New_Track.ogg = New Track\n"));
    }

    #[test]
    fn adds_missing_sections_at_the_end() {
        let out = set_section_key(FILE, "Overlay", "Enabled", "true");
        assert!(out.contains("\n[Overlay]\nEnabled = true\n"));
        let empty = set_section_key("", "Performance", "VSync", "1");
        assert_eq!(empty, "[Performance]\nVSync = 1\n");
    }

    #[test]
    fn removes_keys_from_one_section_only() {
        let out = remove_section_key(FILE, "CDTracks", "Last_Wave.ogg");
        assert!(!out.contains("Last_Wave.ogg"));
        assert!(out.contains("01_Splash_wave.ogg"));
        let out = remove_section_key(&out, "CDTracks", "or2ed1.ogg");
        assert!(!out.contains("or2ed1.ogg"));
        // A commented line is kept.
        let with_comment = "[S]\n# key = 1\nother = 2\n";
        assert_eq!(remove_section_key(with_comment, "S", "key"), with_comment);
    }

    #[test]
    fn lists_section_and_key_pairs() {
        let keys = section_keys(FILE);
        assert!(keys.contains(&("Performance".into(), "FramerateLimit".into())));
        assert!(keys.contains(&("Controls".into(), "VibrationMode".into())));
        assert!(!keys.iter().any(|(_, k)| k.starts_with('#')));
        assert_eq!(keys.len(), 7);
    }

    #[test]
    fn reads_values_back() {
        assert_eq!(
            section_key(FILE, "Performance", "FramerateLimit").as_deref(),
            Some("60")
        );
        assert_eq!(
            section_key(FILE, "Controls", "VibrationMode").as_deref(),
            Some("0")
        );
        assert_eq!(section_key(FILE, "Controls", "Nope"), None);
        assert_eq!(section_key(FILE, "Nope", "FramerateLimit"), None);
    }

    #[test]
    fn keeps_a_file_without_a_trailing_newline_shape() {
        let no_newline = FILE.trim_end();
        let out = set_section_key(no_newline, "Graphics", "DrawDistanceIncrease", "4");
        assert!(out.ends_with("or2ed1.ogg = ending"));
        assert!(!out.ends_with('\n'));
    }
}
