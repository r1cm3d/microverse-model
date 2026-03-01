pub mod dataset;
pub mod generate;
pub mod model;
pub mod tokenizer;
pub mod train;
pub mod tts_bridge;

use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct DialogueLine {
    pub index: usize,
    #[serde(rename = "season no.")]
    pub season: u8,
    #[serde(rename = "episode no.")]
    pub episode_no: u8,
    #[serde(rename = "episode name")]
    pub episode: String,
    #[serde(rename = "name")]
    pub character: String,
    pub line: String,
}

pub fn write_dialogues_to_csv<W: Write>(
    mut writer: W,
    dialogues: &[DialogueLine],
) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(
        writer,
        "index,season no.,episode no.,episode name,name,line"
    )?;

    for dialogue in dialogues {
        let episode = dialogue.episode.replace('"', "'");
        let character = dialogue.character.replace('"', "'");
        let line = dialogue.line.replace('"', "'");

        writeln!(
            writer,
            "{},{},{},\"{}\",\"{}\",\"{}\"",
            dialogue.index, dialogue.season, dialogue.episode_no, episode, character, line
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_dialogues_to_csv() {
        let dialogues = vec![
            DialogueLine {
                index: 1,
                season: 1,
                episode_no: 1,
                episode: "Pilot".to_string(),
                character: "Rick".to_string(),
                line: "Hello".to_string(),
            },
            DialogueLine {
                index: 2,
                season: 1,
                episode_no: 1,
                episode: "Pilot".to_string(),
                character: "Morty".to_string(),
                line: "Hi".to_string(),
            },
        ];

        let mut buffer = Vec::new();
        let result = write_dialogues_to_csv(&mut buffer, &dialogues);
        assert!(result.is_ok());

        let output = String::from_utf8(buffer).unwrap();
        let expected = "index,season no.,episode no.,episode name,name,line\n\
                        1,1,1,\"Pilot\",\"Rick\",\"Hello\"\n\
                        2,1,1,\"Pilot\",\"Morty\",\"Hi\"\n";

        assert_eq!(output, expected);
    }

    #[test]
    fn test_write_dialogues_to_csv_escaping() {
        let dialogues = vec![DialogueLine {
            index: 1,
            season: 1,
            episode_no: 1,
            episode: "The \"Pilot\"".to_string(),
            character: "Rick \"C-137\"".to_string(),
            line: "I said \"Hello\"".to_string(),
        }];

        let mut buffer = Vec::new();
        write_dialogues_to_csv(&mut buffer, &dialogues).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        let expected = "index,season no.,episode no.,episode name,name,line\n\
                        1,1,1,\"The 'Pilot'\",\"Rick 'C-137'\",\"I said 'Hello'\"\n";

        assert_eq!(output, expected);
    }
}
