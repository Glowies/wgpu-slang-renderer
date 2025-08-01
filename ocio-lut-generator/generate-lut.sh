#!/bin/bash
# ociobakelut --inputspace scene_linear --displayview 'sRGB - Display' 'ACES 2.0 - SDR 100 nits (Rec.709)' --format spi3d display-view.spi3d
ociolutimage --generate --cubesize 32 --output clean32.exr
ocioconvert --view clean32.exr scene_linear display-view32.exr 'sRGB - Display' 'ACES 2.0 - SDR 100 nits (Rec.709)'
