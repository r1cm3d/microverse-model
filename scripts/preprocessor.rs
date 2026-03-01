use microverse_model::{write_dialogues_to_csv, DialogueLine};
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_path = "datasets/rick_morty_all_transcripts.csv";
    let output_path = "datasets/rick_morty_transcripts_clean.csv";

    println!("Reading {}", input_path);
    let dialogues = read_dialogues_from_csv(input_path)?;
    println!("Loaded {} dialogue lines", dialogues.len());

    let cleaned: Vec<DialogueLine> = dialogues
        .into_iter()
        .filter_map(clean_dialogue)
        .enumerate()
        .map(|(i, mut d)| {
            d.index = i;
            d
        })
        .collect();

    println!("Cleaned {} dialogue lines", cleaned.len());

    let file = File::create(output_path)?;
    write_dialogues_to_csv(file, &cleaned)?;

    println!("Saved to {}", output_path);

    Ok(())
}

fn clean_dialogue(mut dialogue: DialogueLine) -> Option<DialogueLine> {
    dialogue.line = remove_stage_directions(&dialogue.line);
    dialogue.line = normalize_unicode(&dialogue.line);
    dialogue.line = dialogue.line.trim().to_string();
    dialogue.character = dialogue.character.trim().to_string();

    if dialogue.line.is_empty() || dialogue.line.len() > 1000 {
        return None;
    }

    Some(dialogue)
}

fn remove_stage_directions(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut depth = 0usize;

    for ch in text.chars() {
        match ch {
            '[' => depth += 1,
            ']' if depth > 0 => depth -= 1,
            _ if depth == 0 => result.push(ch),
            _ => {}
        }
    }

    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_unicode(text: &str) -> String {
    text.replace('\u{2014}', "--").replace('\u{2026}', "...")
}

fn read_dialogues_from_csv(path: &str) -> Result<Vec<DialogueLine>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut dialogues = Vec::new();

    for (line_no, line) in reader.lines().enumerate() {
        let line = line?;
        if line_no == 0 || line.is_empty() {
            continue;
        }

        let fields = parse_csv_line(&line);
        if fields.len() != 6 {
            continue;
        }

        let (Ok(index), Ok(season), Ok(episode_no)) = (
            fields[0].parse::<usize>(),
            fields[1].parse::<u8>(),
            fields[2].parse::<u8>(),
        ) else {
            continue;
        };

        dialogues.push(DialogueLine {
            index,
            season,
            episode_no,
            episode: fields[3].clone(),
            character: fields[4].clone(),
            line: fields[5].clone(),
        });
    }

    Ok(dialogues)
}

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in line.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                fields.push(current.clone());
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    fields.push(current);
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_stage_directions_only_bracket() {
        assert_eq!(remove_stage_directions("[gasps]"), "");
        assert_eq!(remove_stage_directions("[Laughs]"), "");
    }

    #[test]
    fn test_remove_stage_directions_inline() {
        assert_eq!(
            remove_stage_directions("Hello [gasps] world"),
            "Hello world"
        );
    }

    #[test]
    fn test_remove_stage_directions_leading() {
        assert_eq!(remove_stage_directions("[gasps] Hello"), "Hello");
    }

    #[test]
    fn test_remove_stage_directions_trailing() {
        assert_eq!(remove_stage_directions("Hello [gasps]"), "Hello");
    }

    #[test]
    fn test_remove_stage_directions_no_brackets() {
        assert_eq!(remove_stage_directions("Hello world"), "Hello world");
    }

    #[test]
    fn test_remove_stage_directions_multiple() {
        assert_eq!(
            remove_stage_directions("[gasps] Hello [laughs] world"),
            "Hello world"
        );
    }

    #[test]
    fn test_normalize_unicode_em_dash() {
        assert_eq!(normalize_unicode("Hello\u{2014}world"), "Hello--world");
    }

    #[test]
    fn test_normalize_unicode_ellipsis() {
        assert_eq!(normalize_unicode("Hello\u{2026}"), "Hello...");
    }

    #[test]
    fn test_normalize_unicode_both() {
        assert_eq!(
            normalize_unicode("Hello\u{2014}world\u{2026}"),
            "Hello--world..."
        );
    }

    #[test]
    fn test_normalize_unicode_no_special() {
        assert_eq!(normalize_unicode("Hello world"), "Hello world");
    }

    #[test]
    fn test_clean_dialogue_removes_stage_directions() {
        let dialogue = DialogueLine {
            index: 0,
            season: 1,
            episode_no: 1,
            episode: "Pilot".to_string(),
            character: "Rick".to_string(),
            line: "[gasps] Hello world".to_string(),
        };
        assert_eq!(clean_dialogue(dialogue).unwrap().line, "Hello world");
    }

    #[test]
    fn test_clean_dialogue_drops_empty_after_cleaning() {
        let dialogue = DialogueLine {
            index: 0,
            season: 1,
            episode_no: 1,
            episode: "Pilot".to_string(),
            character: "Rick".to_string(),
            line: "[gasps]".to_string(),
        };
        assert!(clean_dialogue(dialogue).is_none());
    }

    #[test]
    fn test_clean_dialogue_drops_long_line() {
        let dialogue = DialogueLine {
            index: 0,
            season: 1,
            episode_no: 1,
            episode: "Pilot".to_string(),
            character: "Rick".to_string(),
            line: "a".repeat(1001),
        };
        assert!(clean_dialogue(dialogue).is_none());
    }

    #[test]
    fn test_clean_dialogue_keeps_exactly_1000_chars() {
        let dialogue = DialogueLine {
            index: 0,
            season: 1,
            episode_no: 1,
            episode: "Pilot".to_string(),
            character: "Rick".to_string(),
            line: "a".repeat(1000),
        };
        assert!(clean_dialogue(dialogue).is_some());
    }

    #[test]
    fn test_clean_dialogue_preserves_stuttering() {
        let dialogue = DialogueLine {
            index: 0,
            season: 1,
            episode_no: 1,
            episode: "Pilot".to_string(),
            character: "Morty".to_string(),
            line: "I-I-I don't know Rick".to_string(),
        };
        assert_eq!(
            clean_dialogue(dialogue).unwrap().line,
            "I-I-I don't know Rick"
        );
    }

    #[test]
    fn test_clean_dialogue_trims_whitespace() {
        let dialogue = DialogueLine {
            index: 0,
            season: 1,
            episode_no: 1,
            episode: "Pilot".to_string(),
            character: "  Rick  ".to_string(),
            line: "  Hello  ".to_string(),
        };
        let cleaned = clean_dialogue(dialogue).unwrap();
        assert_eq!(cleaned.line, "Hello");
        assert_eq!(cleaned.character, "Rick");
    }

    #[test]
    fn test_parse_csv_line_simple() {
        let line = r#"0,1,1,"Pilot","Rick","Hello world""#;
        let fields = parse_csv_line(line);
        assert_eq!(fields, vec!["0", "1", "1", "Pilot", "Rick", "Hello world"]);
    }

    #[test]
    fn test_parse_csv_line_with_comma_in_field() {
        let line = r#"0,1,1,"Hello, World","Rick","Well, hello""#;
        let fields = parse_csv_line(line);
        assert_eq!(
            fields,
            vec!["0", "1", "1", "Hello, World", "Rick", "Well, hello"]
        );
    }

    #[test]
    fn test_parse_csv_line_unquoted_integers() {
        let line = r#"42,3,7,"Morty's Mind Blowers","Rick","Boom""#;
        let fields = parse_csv_line(line);
        assert_eq!(fields[0], "42");
        assert_eq!(fields[1], "3");
        assert_eq!(fields[2], "7");
    }
}
