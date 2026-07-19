/// Parse a .osu file to extract the audio filename from the [General] section.
pub fn parse_audio_filename(osu_content: &str) -> Option<String> {
    let mut in_general = false;
    for line in osu_content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let section = &trimmed[1..trimmed.len() - 1];
            in_general = section == "General";
            continue;
        }
        if in_general {
            if let Some((key, value)) = trimmed.split_once(':') {
                if key.trim() == "AudioFilename" {
                    return Some(value.trim().trim_matches('"').to_string());
                }
            }
        }
    }

    None
}

/// Parse a .osu file to extract the background image filename from the [Events] section.
pub fn parse_background_filename(osu_content: &str) -> Option<String> {
    let mut in_events = false;
    for line in osu_content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let section = &trimmed[1..trimmed.len() - 1];
            in_events = section == "Events";
            continue;
        }
        if in_events {
            let parts: Vec<&str> = trimmed.split(',').map(|s| s.trim()).collect();
            if parts.len() >= 3 && parts[0] == "0" && parts[1] == "0" {
                return Some(parts[2].trim_matches('"').to_string());
            }
        }
    }
    None
}

/// Parse a .osu file to extract both the audio filename and background filename in one pass.
pub fn parse_audio_and_background(osu_content: &str) -> (Option<String>, Option<String>) {
    let mut audio = None;
    let mut background = None;
    let mut current_section: Option<&str> = None;

    for line in osu_content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = Some(&trimmed[1..trimmed.len() - 1]);
            continue;
        }
        match current_section {
            Some("General") => {
                if let Some((key, value)) = trimmed.split_once(':') {
                    if key.trim() == "AudioFilename" {
                        audio = Some(value.trim().trim_matches('"').to_string());
                    }
                }
            }
            Some("Events") => {
                let parts: Vec<&str> = trimmed.split(',').map(|s| s.trim()).collect();
                if parts.len() >= 3 && parts[0] == "0" && parts[1] == "0" {
                    background = Some(parts[2].trim_matches('"').to_string());
                    break;
                }
            }
            _ => {}
        }
    }
    (audio, background)
}

#[cfg(test)]
mod tests {
    use super::*;

    const OSU_CONTENT: &str = r#"osu file format v14

[General]
AudioFilename: song.mp3
AudioLeadIn: 0
Mode: 3

[Events]
//Background and Video events
0,0,"bg.jpg",0,0
//Break Periods
2,52831,59432

[Metadata]
Title:Test Song
"#;

    #[test]
    fn parse_audio_filename_found() {
        assert_eq!(parse_audio_filename(OSU_CONTENT), Some("song.mp3".into()));
    }

    #[test]
    fn parse_audio_filename_not_found() {
        assert_eq!(parse_audio_filename("[General]\nOther: value"), None);
    }

    #[test]
    fn parse_background_filename_found() {
        assert_eq!(parse_background_filename(OSU_CONTENT), Some("bg.jpg".into()));
    }

    #[test]
    fn parse_background_filename_not_found() {
        assert_eq!(parse_background_filename("[Events]\n1,0,\"bg.jpg\""), None);
    }

    #[test]
    fn parse_audio_and_background_both_found() {
        let (audio, bg) = parse_audio_and_background(OSU_CONTENT);
        assert_eq!(audio, Some("song.mp3".into()));
        assert_eq!(bg, Some("bg.jpg".into()));
    }

    #[test]
    fn parse_audio_and_background_empty() {
        let (audio, bg) = parse_audio_and_background("");
        assert_eq!(audio, None);
        assert_eq!(bg, None);
    }

    #[test]
    fn parse_with_comments_skipped() {
        let content = "[General]\n// comment\nAudioFilename: music.ogg\n// another comment";
        assert_eq!(parse_audio_filename(content), Some("music.ogg".into()));
    }

    #[test]
    fn parse_with_quoted_values() {
        let content = "[General]\nAudioFilename: \"quoted song.mp3\"";
        assert_eq!(parse_audio_filename(content), Some("quoted song.mp3".into()));
    }

    #[test]
    fn background_with_quotes() {
        let content = "[Events]\n0,0,\"background image.png\"";
        assert_eq!(parse_background_filename(content), Some("background image.png".into()));
    }
}
