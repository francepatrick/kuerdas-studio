# Original decoder fixtures

These tiny files contain an original 330 Hz sine wave, generated locally for decoder tests. They contain no commercial music or user recordings. Tests need no external encoder.

Regenerate with FFmpeg (44100 Hz, 0.15 seconds):

```sh
ffmpeg -f lavfi -i 'sine=frequency=330:sample_rate=44100:duration=0.15' -ac 1 -c:a libmp3lame -b:a 64k tone.mp3
ffmpeg -f lavfi -i 'sine=frequency=330:sample_rate=44100:duration=0.15' -ac 1 -c:a flac tone.flac
ffmpeg -f lavfi -i 'sine=frequency=330:sample_rate=44100:duration=0.15' -ac 1 -c:a aac -b:a 64k tone.m4a
ffmpeg -f lavfi -i 'sine=frequency=330:sample_rate=44100:duration=0.15' -ac 2 -c:a vorbis -strict experimental tone.ogg
```
