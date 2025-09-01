#!/bin/bash
TEMP_DIR="./temp"
OCIO_CONFIG="../studio-config-all-views-v2.3.0_aces-v2.0_ocio-v2.4.ocio"

LUT_SHAPER_SPACE="acescct_ap1"
# OUT_DISPLAY="sRGB - Display"
OUT_DISPLAY="Display P3 - Display"
# OUT_VIEW="Raw"
# OUT_VIEW="Un-tone-mapped"
# OUT_VIEW="ACES 2.0 - SDR 100 nits (Rec.709)"
OUT_VIEW="ACES 2.0 - SDR 100 nits (P3 D65)"

INTER_FORMAT="exr"
NUM_SPLITS=$1
OUTPUT_KTX=$2
CLEAN_EXR_LUT=${TEMP_DIR}/clean${NUM_SPLITS}.exr
FULL_EXR_LUT=${TEMP_DIR}/shaper-to-display-view${NUM_SPLITS}.exr
OUTPUT_PREFIX="strip"

if [ -z "$1" ]; then
  echo "ERROR: Argument 1 is missing. Argument 1 should be the resolution of the cubemap."
  exit 1
fi

if [ -z "$2" ]; then
  echo "ERROR: Argument 2 is missing. Argument 2 should be the output ktx2 file path."
  exit 1
fi

IMAGE_WIDTH=$(($NUM_SPLITS * $NUM_SPLITS))
IMAGE_HEIGHT=$NUM_SPLITS
STRIP_WIDTH=$NUM_SPLITS

# make temp dir
mkdir $TEMP_DIR

# ociobakelut --inputspace lin_ap1_shaper --displayview 'sRGB - Display' 'ACES 2.0 - SDR 100 nits (Rec.709)' --cubesize 65 --format resolve_cube display-view.cube
ociolutimage --generate --cubesize $NUM_SPLITS --maxwidth $IMAGE_WIDTH --output $CLEAN_EXR_LUT --config $OCIO_CONFIG
ocioconvert --view $CLEAN_EXR_LUT "$LUT_SHAPER_SPACE" $FULL_EXR_LUT "$OUT_DISPLAY" "$OUT_VIEW" --iconfig $OCIO_CONFIG

echo "  Each strip will be ${STRIP_WIDTH} pixels wide."
echo "  Starting image splitting..."

ALL_SPLITS=""

for i in $(seq 0 $((NUM_SPLITS - 1))); do
    # Calculate the starting Y-coordinate for the current strip
    X_START=$((i * STRIP_WIDTH))

    # Construct the output filename with zero-padding for consistent naming
    OUTPUT_IMAGE="${TEMP_DIR}/${OUTPUT_PREFIX}_$(printf "%02d" "$i").${INTER_FORMAT}"
    ALL_SPLITS="${ALL_SPLITS} ${OUTPUT_IMAGE}"

    oiiotool "${FULL_EXR_LUT}" --cut "${STRIP_WIDTH}x${STRIP_WIDTH}+${X_START}+0}" -d half -o "${OUTPUT_IMAGE}"

    echo "  Created ${OUTPUT_IMAGE}"
done

echo "  Image splitting complete! ${NUM_SPLITS} horizontal strips created."

# generate KTX2 image from the split images
# we need to swizzle to swap the green and blue channels as this
# is what is expected by the tone mapping shader
ktx create --input-swizzle rbg1 --format E5B9G9R9_UFLOAT_PACK32 --zstd 20 --depth $NUM_SPLITS $ALL_SPLITS $OUTPUT_KTX

# clean up
rm -rf $TEMP_DIR
