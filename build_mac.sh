#!/bin/sh

if [[ $# -eq 0 ]] ; then
    echo 'Usage: build_mac.sh [build-number]'
    exit 1
fi

export $(grep -v '^#' .env | xargs)

set -e

# Compile for older macOS Versions
# See: https://users.rust-lang.org/t/compile-rust-binary-for-older-versions-of-mac-osx/38695/2
export MACOSX_DEPLOYMENT_TARGET=10.14

rm -rf target/release/bundle/osx/Ebou.app
rm -rf target/release/bundle/osx/Ebou.app.dSYM

# Build for x86 and ARM
cargo build --release --target=aarch64-apple-darwin
cargo build --release --target=x86_64-apple-darwin

# Combine into a fat binary

lipo -create target/aarch64-apple-darwin/release/ebou target/x86_64-apple-darwin/release/ebou -output ebou

# copy it to where cargo bundle expects it
mkdir target/release || true
cp ebou target/release/

# Perform Cargo bundle to create a macOS Bundle

cargo bundle --release

# Override bundle binary with the fat one

rm target/release/bundle/osx/Ebou.app/Contents/MacOS/ebou

mv ./ebou target/release/bundle/osx/Ebou.app/Contents/MacOS/Ebou

# Tell the Info.plist or binary is capitalized

/usr/libexec/PlistBuddy -c "Set :CFBundleExecutable Ebou" "target/release/bundle/osx/Ebou.app/Contents/Info.plist"

# Set the bundle version
echo "BUNDLE VERSION $1"
/usr/libexec/PlistBuddy -c "Set :CFBundleVersion $1" "target/release/bundle/osx/Ebou.app/Contents/Info.plist"

# Add a lot of Xcode Flags into the plist
echo "Still setting to Xcode 14.2 (14C18)"
# FIXME: Get these values from environment
/usr/libexec/PlistBuddy -c "Add :DTCompiler string com.apple.compilers.llvm.clang.1_0" "target/release/bundle/osx/Ebou.app/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Add :DTPlatformBuild string 14C18" "target/release/bundle/osx/Ebou.app/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Add :DTPlatformName string macosx" "target/release/bundle/osx/Ebou.app/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Add :DTPlatformVersion string 13.1" "target/release/bundle/osx/Ebou.app/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Add :DTSDKBuild string 22C55" "target/release/bundle/osx/Ebou.app/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Add :DTSDKName string macosx13.1" "target/release/bundle/osx/Ebou.app/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Add :DTXcode string 1420" "target/release/bundle/osx/Ebou.app/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Add :DTXcodeBuild string 14C18" "target/release/bundle/osx/Ebou.app/Contents/Info.plist"

dsymutil target/release/ebou
mv target/release/ebou.dSYM target/release/bundle/osx/Ebou.app.dSYM

rm target/release/ebou

cp ./resources/PkgInfo target/release/bundle/osx/Ebou.app/Contents/

# Create a zip file
echo "Created target/release/bundle/osx/"