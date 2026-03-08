#!/bin/sh
set -e
cd target/dx/*/release/android/app/ || exit
# Clean and replace Icons, then build
./gradlew clean
# rm --recursive --verbose app/src/main/res/mipmap*
cp --recursive --verbose ../../../../../../android/res app/src/main/
./gradlew assembleRelease
