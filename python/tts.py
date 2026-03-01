import argparse
import os
import sys


def main():
    parser = argparse.ArgumentParser(description="XTTS-v2 voice synthesis")
    parser.add_argument("--text", required=True, help="Text to synthesize")
    parser.add_argument("--speaker", required=True, choices=["rick", "morty"], help="Speaker name")
    parser.add_argument("--output", required=True, help="Output WAV file path")
    parser.add_argument("--samples-dir", default="audio_samples", help="Directory with reference WAV files")
    args = parser.parse_args()

    ref_path = os.path.join(args.samples_dir, f"{args.speaker}_reference.wav")
    if not os.path.isfile(ref_path):
        print(f"Error: reference file not found: {ref_path}", file=sys.stderr)
        sys.exit(1)

    from TTS.api import TTS

    tts = TTS(model_name="tts_models/multilingual/multi-dataset/xtts_v2")
    tts.tts_to_file(text=args.text, speaker_wav=ref_path, language="en", file_path=args.output)

    if not os.path.isfile(args.output):
        print(f"Error: output file was not created: {args.output}", file=sys.stderr)
        sys.exit(1)

    print(args.output)


if __name__ == "__main__":
    main()
