#!/bin/bash

set -euo pipefail

cargo build --release

rm -rf ./spaceport.app
mkdir -p ./spaceport.app/Contents/MacOS/
mkdir -p ./spaceport.app/Contents/Resources/

cp ./target/release/spaceport ./spaceport.app/Contents/MacOS/spaceport
cp ./assets/icon.icns ./spaceport.app/Contents/Resources/icon.icns
cp info.plist ./spaceport.app/Contents/Info.plist
codesign --force --deep --sign - ./spaceport.app
