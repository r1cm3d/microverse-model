use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
struct DialogueLine {
    index: usize,
    #[serde(rename = "season no.")]
    season: u8,
    #[serde(rename = "episode no.")]
    episode_no: u8,
    #[serde(rename = "episode name")]
    episode: String,
    #[serde(rename = "name")]
    character: String,
    line: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Rick and Morty transcript scraper...");

    let mut all_dialogues = Vec::new();
    let mut current_index = 2488; // Starting index after Season 3 (from rick_and_morty.csv)

    // Scrape seasons 4 to 8
    for season in 4..=8 {
        println!("\n=== Scraping Season {} ===", season);
        let episodes = get_episode_list(season).await?;

        for (episode_no, episode_name, episode_url) in episodes {
            println!("Scraping S{}E{}: {}", season, episode_no, episode_name);
            match scrape_episode(
                &episode_url,
                season,
                episode_no,
                &episode_name,
                current_index,
            )
            .await
            {
                Ok(mut dialogues) => {
                    let count = dialogues.len();
                    current_index += count;
                    all_dialogues.append(&mut dialogues);
                    println!("  ✓ Got {} lines", count);
                }
                Err(e) => {
                    eprintln!("  ✗ Error scraping {}: {}", episode_name, e);
                }
            }

            // Be respectful - add delay between requests
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }

    // Save to JSON
    let json = serde_json::to_string_pretty(&all_dialogues)?;
    let mut file = File::create("datasets/rick_morty_s4_s8_transcripts.json")?;
    file.write_all(json.as_bytes())?;

    // Save to CSV
    save_to_csv(&all_dialogues, "datasets/rick_morty_s4_s8_transcripts.csv")?;

    println!("\n=== Complete ===");
    println!("Total lines scraped: {}", all_dialogues.len());
    println!("Saved to: datasets/rick_morty_s4_s8_transcripts.json and .csv");

    Ok(())
}

async fn get_episode_list(
    season: u8,
) -> Result<Vec<(u8, String, String)>, Box<dyn std::error::Error>> {
    let url = format!("https://rickandmorty.fandom.com/wiki/Season_{}", season);

    let response = reqwest::get(&url).await?;
    let body = response.text().await?;
    let document = Html::parse_document(&body);

    let table_selector = Selector::parse("table").unwrap();
    let row_selector = Selector::parse("tr").unwrap();
    let cell_selector = Selector::parse("td").unwrap();
    let link_selector = Selector::parse("a").unwrap();
    // Some tables might have headers in the first row

    let mut episodes = Vec::new();

    for table in document.select(&table_selector) {
        let rows = table.select(&row_selector);

        for row in rows {
            let cells: Vec<_> = row.select(&cell_selector).collect();
            if cells.len() < 3 {
                continue;
            }

            // Try to parse the first cell as episode number
            // Column 0: No. in season
            // Column 1: No. overall
            // Column 2: Title

            let episode_no_str = cells[0].text().collect::<String>().trim().to_string();
            // Remove double quotes or special chars if any
            let episode_no_clean = episode_no_str.replace("\"", "");
            let episode_no = match episode_no_clean.parse::<u8>() {
                Ok(n) => n,
                Err(_) => continue, // Skip header or non-episode rows
            };

            // Extract title and link from the 3rd cell (index 2)
            if let Some(link) = cells[2].select(&link_selector).next() {
                let title = link
                    .value()
                    .attr("title")
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| link.text().collect::<String>());
                let href = link.value().attr("href").unwrap_or("");

                if !href.is_empty() {
                    let full_url = format!("https://rickandmorty.fandom.com{}/Transcript", href);
                    // Remove " (episode)" from title if present, just in case
                    let clean_title = title.replace(" (episode)", "");
                    episodes.push((episode_no, clean_title, full_url));
                }
            }
        }

        if !episodes.is_empty() {
            break;
        }
    }

    Ok(episodes)
}

async fn scrape_episode(
    url: &str,
    season: u8,
    episode_no: u8,
    episode_name: &str,
    start_index: usize,
) -> Result<Vec<DialogueLine>, Box<dyn std::error::Error>> {
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    let document = Html::parse_document(&body);

    let mut dialogues = Vec::new();
    let mut current_index = start_index;

    // The transcript is usually in a div with class "mw-parser-output"
    let content_selector = Selector::parse("div.mw-parser-output").unwrap();
    let p_selector = Selector::parse("p, dl, dd").unwrap();

    if let Some(content) = document.select(&content_selector).next() {
        for element in content.select(&p_selector) {
            let text = element.text().collect::<String>().trim().to_string();

            if text.is_empty() {
                continue;
            }

            if let Some((character, line)) = parse_dialogue_line(&text) {
                dialogues.push(DialogueLine {
                    index: current_index,
                    season,
                    episode_no,
                    episode: episode_name.to_string(),
                    character,
                    line,
                });
                current_index += 1;
            }
        }
    }

    Ok(dialogues)
}

fn parse_dialogue_line(text: &str) -> Option<(String, String)> {
    if let Some(colon_pos) = text.find(':') {
        let character = text[..colon_pos].trim().to_string();
        let line = text[colon_pos + 1..].trim().to_string();

        if !character.is_empty() && character.len() < 50 && !line.is_empty() {
            return Some((character, line));
        }
    }
    None
}

fn save_to_csv(
    dialogues: &[DialogueLine],
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(filename)?;

    // Write header matching rick_and_morty.csv
    writeln!(file, "index,season no.,episode no.,episode name,name,line")?;

    // Write data
    for dialogue in dialogues {
        // Replace double quotes with single quotes as requested
        let episode = dialogue.episode.replace("\"", "'");
        let character = dialogue.character.replace("\"", "'");
        let line = dialogue.line.replace("\"", "'");

        // Manually write CSV line to ensure quoting of string fields
        writeln!(
            file,
            "{},{},{},\"{}\",\"{}\",\"{}\"",
            dialogue.index, dialogue.season, dialogue.episode_no, episode, character, line
        )?;
    }

    Ok(())
}
