#!/bin/bash
TEMP_DIR="./temp"
INPUT_EXR=$1
OUTPUT_KTX=$2
FACE_PREFIX="face"

if [ -z "$1" ]; then
  echo "ERROR: Argument 1 is missing. Argument 1 should be the input .exr file with long-lat equirectangular projection."
  exit 1
fi

if [ -z "$2" ]; then
  echo "ERROR: Argument 2 is missing. Argument 2 should be the output .ktx2 file."
  exit 1
fi

mkdir $TEMP_DIR
mkdir $TEMP_DIR/tiled
mkdir $TEMP_DIR/scanline

TEMP_CUBE_FACE="${TEMP_DIR}/tiled/${FACE_PREFIX}%.exr"
exrenvmap -w 256 $INPUT_EXR $TEMP_CUBE_FACE

# output from `exrenvmap` are tiled exr files but `ktx create` gets a segfault
# with tiled exrs. So we need to convert the exrs to scanline instead
for file in ${TEMP_DIR}/tiled/*
do
  FILE_NAME=$(basename "$file")
  OUT_PATH="${TEMP_DIR}/scanline/${FILE_NAME}"
  oiiotool $file --scanline -o $OUT_PATH
done

CUBE_FACES="${TEMP_DIR}/scanline/*"
ktx create --format R16G16B16A16_SFLOAT --zstd 20 --assign-primaries srgb --assign-tf linear --cubemap $CUBE_FACES $OUTPUT_KTX

rm -rf $TEMP_DIR
