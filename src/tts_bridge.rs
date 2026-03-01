use anyhow::Context;
use std::path::PathBuf;
use std::process::Command;

pub struct TtsBridgeConfig {
    pub text: String,
    pub character: String,
    pub output: PathBuf,
    pub script: PathBuf,
    pub samples_dir: PathBuf,
    pub python: String,
}

pub fn speak(config: TtsBridgeConfig) -> anyhow::Result<PathBuf> {
    let speaker = config.character.to_lowercase();

    eprintln!("Synthesizing audio (this may take ~30-60s on CPU)...");

    let output = Command::new(&config.python)
        .arg(&config.script)
        .arg("--text")
        .arg(&config.text)
        .arg("--speaker")
        .arg(&speaker)
        .arg("--output")
        .arg(&config.output)
        .arg("--samples-dir")
        .arg(&config.samples_dir)
        .output()
        .with_context(|| format!("failed to spawn {}", config.python))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("tts.py exited with error:\n{}", stderr);
    }

    if !config.output.exists() {
        anyhow::bail!(
            "tts.py exited 0 but output file not found: {}",
            config.output.display()
        );
    }

    Ok(config.output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speak_missing_python_returns_error() {
        let config = TtsBridgeConfig {
            text: "test".to_string(),
            character: "RICK".to_string(),
            output: PathBuf::from("/tmp/test_nonexistent_output.wav"),
            script: PathBuf::from("python/tts.py"),
            samples_dir: PathBuf::from("audio_samples"),
            python: "nonexistent_python_binary_xyz".to_string(),
        };
        let result = speak(config);
        assert!(result.is_err());
    }
}
