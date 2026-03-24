#!/usr/bin/env bash

# ============================================================================
# Deflicker an LED matrix recording using ffmpeg
# ============================================================================

INPUT_FILE="/Users/userfriendly/Downloads/A001_03241852_C003.MOV"

OUTPUT_FILE="${INPUT_FILE%.*}_deflickered.mp4"

ffmpeg -i "$INPUT_FILE" \
  -vf "deflicker=size=15:mode=qm" \
  -c:a copy \
  "$OUTPUT_FILE"

echo "Done: $OUTPUT_FILE"
