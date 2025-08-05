#!/bin/bash
TEMP_DIR="./temp"
OCIO="./studio-config-all-views-v2.3.0_aces-v2.0_ocio-v2.4.ocio"
INTER_FORMAT="png"
CLEAN_EXR_LUT=${TEMP_DIR}/clean32.${INTER_FORMAT}
FULL_EXR_LUT=${TEMP_DIR}/shaper-to-display-view32.${INTER_FORMAT}
OUTPUT_PREFIX="strip"
NUM_SPLITS=32                      

IMAGE_WIDTH=$(($NUM_SPLITS * $NUM_SPLITS))
IMAGE_HEIGHT=$NUM_SPLITS
STRIP_WIDTH=$NUM_SPLITS

# make temp dir
mkdir $TEMP_DIR

# ociobakelut --inputspace lin_ap1_shaper --displayview 'sRGB - Display' 'ACES 2.0 - SDR 100 nits (Rec.709)' --cubesize 65 --format resolve_cube display-view.cube
ociolutimage --generate --cubesize 32 --output $CLEAN_EXR_LUT
ocioconvert --view $CLEAN_EXR_LUT lin_ap1_shaper $FULL_EXR_LUT 'sRGB - Display' 'ACES 2.0 - SDR 100 nits (Rec.709)'

echo "  Each strip will be ${STRIP_WIDTH} pixels wide."
echo "  Starting image splitting..."

for i in $(seq 0 $((NUM_SPLITS - 1))); do
    # Calculate the starting Y-coordinate for the current strip
    X_START=$((i * STRIP_WIDTH))

    # Construct the output filename with zero-padding for consistent naming
    OUTPUT_IMAGE="${OUTPUT_PREFIX}_$(printf "%02d" "$i").${INTER_FORMAT}"

    oiiotool "${FULL_EXR_LUT}" --cut "${STRIP_WIDTH}x${STRIP_WIDTH}+${X_START}+0}" -o "${TEMP_DIR}/${OUTPUT_IMAGE}"

    echo "  Created ${OUTPUT_IMAGE} (Y-range: ${Y_START} to $((Y_END - 1)))"
done

echo "  Image splitting complete! ${NUM_SPLITS} horizontal strips created."

# rm -rf $TEMP_DIR
