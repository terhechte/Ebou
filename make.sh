#!/bin/sh

#!/bin/sh
if [[ $# -ne 3 ]] ; then
    echo 'Usage: make.sh [signing identity] [version-number] [path-to-xcarchive]'
    exit 1
fi

# Build

./build_mac.sh "$2"

# Sign

codesign --options runtime -f --timestamp --entitlements resources/entitlements.plist -s "$1" target/release/bundle/osx/Ebou.app

# Copy Over

cp -R "target/release/bundle/osx/Ebou.app.dSYM" "$3/dSYMs/"
cp -R "target/release/bundle/osx/Ebou.app" "$3/Products/Applications/"
