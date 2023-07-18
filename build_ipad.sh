# not really working yet. just building for simulator
cargo build --target aarch64-apple-ios-sim
cp target/aarch64-apple-ios-sim/debug/libebou.a gen/apple/libebou.a