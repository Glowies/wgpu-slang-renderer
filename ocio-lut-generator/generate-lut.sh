#!/bin/bash
TEMP_DIR="./temp"
OCIO_CONFIG="./studio-config-all-views-v2.3.0_aces-v2.0_ocio-v2.4.ocio"
INTER_FORMAT="png"
CLEAN_EXR_LUT=${TEMP_DIR}/clean32.exr
FULL_EXR_LUT=${TEMP_DIR}/shaper-to-display-view32.exr
OUTPUT_PREFIX="strip"
NUM_SPLITS=32                      

IMAGE_WIDTH=$(($NUM_SPLITS * $NUM_SPLITS))
IMAGE_HEIGHT=$NUM_SPLITS
STRIP_WIDTH=$NUM_SPLITS

# make temp dir
mkdir $TEMP_DIR

# ociobakelut --inputspace lin_ap1_shaper --displayview 'sRGB - Display' 'ACES 2.0 - SDR 100 nits (Rec.709)' --cubesize 65 --format resolve_cube display-view.cube
ociolutimage --generate --cubesize 32 --output $CLEAN_EXR_LUT --config $OCIO_CONFIG
ocioconvert --view $CLEAN_EXR_LUT lin_ap1_shaper $FULL_EXR_LUT 'sRGB - Display' 'ACES 2.0 - SDR 100 nits (Rec.709)' --iconfig $OCIO_CONFIG

echo "  Each strip will be ${STRIP_WIDTH} pixels wide."
echo "  Starting image splitting..."

ALL_SPLITS=""

for i in $(seq 0 $((NUM_SPLITS - 1))); do
    # Calculate the starting Y-coordinate for the current strip
    X_START=$((i * STRIP_WIDTH))

    # Construct the output filename with zero-padding for consistent naming
    OUTPUT_IMAGE="${TEMP_DIR}/${OUTPUT_PREFIX}_$(printf "%02d" "$i").${INTER_FORMAT}"
    ALL_SPLITS="${ALL_SPLITS} ${OUTPUT_IMAGE}"

    oiiotool "${FULL_EXR_LUT}" --cut "${STRIP_WIDTH}x${STRIP_WIDTH}+${X_START}+0}" -d uint16 -o "${OUTPUT_IMAGE}"

    echo "  Created ${OUTPUT_IMAGE} (Y-range: ${Y_START} to $((Y_END - 1)))"
done

echo "  Image splitting complete! ${NUM_SPLITS} horizontal strips created."

# generate KTX2 image from the split images
toktx --assign_oetf linear --zcmp --t2 --depth $NUM_SPLITS shaper_to_display32.ktx2 $ALL_SPLITS

# clean up
rm -rf $TEMP_DIR
