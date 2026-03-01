# Reference Audio Guide for XTTS-v2 Voice Cloning

## Overview

XTTS-v2 voice cloning requires a short reference WAV clip (3–15 seconds) per speaker. The clip must be:

- A single speaker with no overlap
- Free of background music and sound effects
- Clear and intelligible throughout

Quality here directly determines output voice similarity. A noisy or music-backed clip will produce a muddied clone regardless of model quality.

---

## Tools Needed

Install on Arch Linux:

```bash
sudo pacman -S yt-dlp ffmpeg
```

---

## Download and Trim Workflow

### 1. Download audio from YouTube

```bash
yt-dlp -x --audio-format wav -o "%(title)s.%(ext)s" "<YouTube-URL>"
```

This downloads the video, strips the audio track, and saves it as a WAV file.

### 2. Trim to the target segment

```bash
ffmpeg -i input.wav -ss 00:01:23 -to 00:01:35 -ar 22050 -ac 1 audio_samples/rick_reference.wav
```

Flags:
- `-ss` — start timestamp
- `-to` — end timestamp
- `-ar 22050` — resample to 22050 Hz (required by XTTS-v2)
- `-ac 1` — convert to mono

### 3. Verify the output

```bash
ffprobe audio_samples/rick_reference.wav
```

Confirm: `Audio: pcm_s16le, 22050 Hz, mono`.

---

## Recommended Source Clips

Use the official Adult Swim / Rick and Morty YouTube channel (`@RickandMorty`) as the primary source — uploads are high-quality and free of re-encoding artifacts from third-party uploads.

### Rick Sanchez

| Episode | Scene | Why it works |
|---------|-------|--------------|
| S01E01 | Portal gun explanation to Morty | Long uninterrupted monologue, no music, clean studio audio |
| S02E01 | Rick's internal monologue setup | Dry delivery, minimal background noise |

Avoid: any clip with the sci-fi electronic score underneath, burp-heavy lines, or crowd/alien ambiance.

### Morty Smith

| Episode | Scene | Why it works |
|---------|-------|--------------|
| S01E06 | Morty's calm explanation of his plan | Normal speaking voice, no panic or screaming |
| S01E02 | Morty and Rick talking quietly in the garage | Conversational pace, clean |

Avoid: screaming compilations, emotional breakdown clips, or scenes with Rick talking over him.

---

## Quality Checklist

Before using a clip, confirm all of the following:

- [ ] Single speaker only — no overlapping voices
- [ ] No background music or sound effects
- [ ] Duration between 3 and 15 seconds
- [ ] Resampled to 22050 Hz mono WAV
- [ ] Speaker is clearly audible throughout the entire clip

---

## Output Paths

Place finished clips at:

```
audio_samples/rick_reference.wav
audio_samples/morty_reference.wav
```

These paths are already listed in `.gitignore` since audio files are not committed to the repository.
