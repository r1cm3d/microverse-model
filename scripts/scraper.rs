use reqwest;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use tokio;

#[derive(Debug, Serialize, Deserialize)]
struct DialogueLine {
    season: u8,
    episode: String,
    character: String,
    line: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Rick and Morty transcript scraper...");
    
    let mut all_dialogues = Vec::new();
    
    // Scrape seasons 6, 7, and 8
    for season in 6..=8 {
        println!("\n=== Scraping Season {} ===", season);
        let episodes = get_episode_list(season).await?;
        
        for (episode_name, episode_url) in episodes {
            println!("Scraping: {}", episode_name);
            match scrape_episode(&episode_url, season, &episode_name).await {
                Ok(mut dialogues) => {
                    all_dialogues.append(&mut dialogues);
                    println!("  ✓ Got {} lines", dialogues.len());
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
    let mut file = File::create("rick_morty_s6_s8_transcripts.json")?;
    file.write_all(json.as_bytes())?;
    
    // Save to CSV
    save_to_csv(&all_dialogues, "rick_morty_s6_s8_transcripts.csv")?;
    
    println!("\n=== Complete ===");
    println!("Total lines scraped: {}", all_dialogues.len());
    println!("Saved to: rick_morty_s6_s8_transcripts.json and .csv");
    
    Ok(())
}

async fn get_episode_list(season: u8) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let url = format!(
        "https://rickandmorty.fandom.com/wiki/Category:Season_{}_transcripts",
        season
    );
    
    let response = reqwest::get(&url).await?;
    let body = response.text().await?;
    let document = Html::parse_document(&body);
    
    // Select episode links from the category page
    let link_selector = Selector::parse("a.category-page__member-link").unwrap();
    
    let mut episodes = Vec::new();
    for element in document.select(&link_selector) {
        if let Some(href) = element.value().attr("href") {
            let title = element.text().collect::<String>();
            let full_url = format!("https://rickandmorty.fandom.com{}", href);
            episodes.push((title, full_url));
        }
    }
    
    Ok(episodes)
}

async fn scrape_episode(
    url: &str,
    season: u8,
    episode_name: &str,
) -> Result<Vec<DialogueLine>, Box<dyn std::error::Error>> {
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    let document = Html::parse_document(&body);
    
    let mut dialogues = Vec::new();
    
    // The transcript is usually in a div with class "mw-parser-output"
    let content_selector = Selector::parse("div.mw-parser-output").unwrap();
    let p_selector = Selector::parse("p, dl, dd").unwrap();
    
    if let Some(content) = document.select(&content_selector).next() {
        for element in content.select(&p_selector) {
            let text = element.text().collect::<String>().trim().to_string();
            
            if text.is_empty() {
                continue;
            }
            
            // Parse character dialogue (format: "CHARACTER: dialogue text")
            if let Some((character, line)) = parse_dialogue_line(&text) {
                dialogues.push(DialogueLine {
                    season,
                    episode: episode_name.to_string(),
                    character,
                    line,
                });
            }
        }
    }
    
    Ok(dialogues)
}

fn parse_dialogue_line(text: &str) -> Option<(String, String)> {
    // Look for pattern "CHARACTER: dialogue"
    if let Some(colon_pos) = text.find(':') {
        let character = text[..colon_pos].trim().to_string();
        let line = text[colon_pos + 1..].trim().to_string();
        
        // Filter out likely false positives (too long character names, etc.)
        if !character.is_empty() && character.len() < 50 && !line.is_empty() {
            return Some((character, line));
        }
    }
    None
}

fn save_to_csv(dialogues: &[DialogueLine], filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_path(filename)?;
    
    // Write header
    wtr.write_record(&["season", "episode", "character", "line"])?;
    
    // Write data
    for dialogue in dialogues {
        wtr.write_record(&[
            dialogue.season.to_string(),
            dialogue.episode.clone(),
            dialogue.character.clone(),
            dialogue.line.clone(),
        ])?;
    }
    
    wtr.flush()?;
    Ok(())
}
