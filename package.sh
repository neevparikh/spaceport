#!/bin/bash

cargo build --release

mkdir -p ./Spaceport.app/Contents/MacOS/ 
mkdir -p ./Spaceport.app/Contents/Resources/

cp ./target/release/spaceport ./Spaceport.app/Contents/MacOS/spaceport
cp ./assets/icon.icns ./Spaceport.app/Contents/Resources/icon.icns
cp info.plist ./Spaceport.app/Contents/Info.plist
