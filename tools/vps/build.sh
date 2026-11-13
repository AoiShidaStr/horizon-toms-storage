#!/bin/bash
# Run on the VPS: tests, the dedicated-server engine (no default features),
# then the client flavour check (default features).
. ~/.cargo/env
export CARGO_TARGET_DIR=/opt/build/target CARGO_BUILD_JOBS=4
WT=/opt/build/wt/toms_storage
cd $WT
date
echo "== tests"; nice -n 10 cargo test -p mod-toms_storage --message-format short 2>&1 | grep -vE "^\s+(Compiling|Checking)" | grep -E "test result|FAILED|panicked|error|warning"; echo "test_exit=${PIPESTATUS[0]}"
echo "== server engine"; nice -n 10 cargo build -p horizon-native-game --no-default-features --message-format short 2>&1 | grep -vE "^\s+(Compiling|Checking)"; echo "build_exit=${PIPESTATUS[0]}"
echo "== client flavour check"; nice -n 10 cargo check -p horizon-native-game --message-format short 2>&1 | grep -vE "^\s+(Compiling|Checking)"; echo "check_exit=${PIPESTATUS[0]}"
date; echo DONE
