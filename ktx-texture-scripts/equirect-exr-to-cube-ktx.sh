#!/bin/bash
TEMP_DIR="./temp"
CUBE_RESOLUTION=$1
INPUT_EXR=$2
OUTPUT_KTX=$3
FACE_PREFIX="face"

if [ -z "$1" ]; then
  echo "ERROR: Argument 1 is missing. Argument 1 should be the resolution of the cubemap."
  exit 1
fi

if [ -z "$2" ]; then
  echo "ERROR: Argument 2 is missing. Argument 2 should be the input .exr file with long-lat equirectangular projection."
  exit 1
fi

if [ -z "$3" ]; then
  echo "ERROR: Argument 3 is missing. Argument 3 should be the output .ktx2 file."
  exit 1
fi

mkdir $TEMP_DIR
mkdir $TEMP_DIR/tiled
mkdir $TEMP_DIR/scanline

TEMP_CUBE_FACE="${TEMP_DIR}/tiled/${FACE_PREFIX}%.exr"
exrenvmap -w $CUBE_RESOLUTION $INPUT_EXR $TEMP_CUBE_FACE

# output from `exrenvmap` are tiled exr files but `ktx create` gets a segfault
# with tiled exrs. So we need to convert the exrs to scanline instead
for file in ${TEMP_DIR}/tiled/*
do
  FILE_NAME=$(basename "$file")
  OUT_PATH="${TEMP_DIR}/scanline/${FILE_NAME}"
  oiiotool $file --scanline -o $OUT_PATH
done

CUBE_FACES="${CUBE_FACES} ${TEMP_DIR}/scanline/${FACE_PREFIX}+X.exr"
CUBE_FACES="${CUBE_FACES} ${TEMP_DIR}/scanline/${FACE_PREFIX}-X.exr"
CUBE_FACES="${CUBE_FACES} ${TEMP_DIR}/scanline/${FACE_PREFIX}+Y.exr"
CUBE_FACES="${CUBE_FACES} ${TEMP_DIR}/scanline/${FACE_PREFIX}-Y.exr"
CUBE_FACES="${CUBE_FACES} ${TEMP_DIR}/scanline/${FACE_PREFIX}-Z.exr"
CUBE_FACES="${CUBE_FACES} ${TEMP_DIR}/scanline/${FACE_PREFIX}+Z.exr"
ktx create --format R16G16B16A16_SFLOAT --zstd 20 --assign-primaries srgb --assign-tf linear --cubemap $CUBE_FACES $OUTPUT_KTX

# generate and store Spherical Harmonics coefficients for the cubemap
cargo run -r -p sh-coefficient-baker -- $CUBE_FACES ${OUTPUT_KTX}.bin

rm -rf $TEMP_DIR
