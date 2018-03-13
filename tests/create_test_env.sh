#!/bin/sh

FOLDER=$(git log --format="%h" -n 1)
mkdir $FOLDER
cargo build --release
cp ../target/release/pill $FOLDER/pill
cargo clean


