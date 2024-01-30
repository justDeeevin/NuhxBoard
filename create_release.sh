#!/bin/bash

echo "Release version: "
read VERSION

cargo build --release
rm -rf release
mkdir release
cp  target/release/NuhxBoard release
cp -r ~/.local/share/NuhxBoard/keyboards release
cp install.sh release
cd release
tar -czvf "../NuhxBoard_v${VERSION}.tar.gz" install.sh NuhxBoard keyboards
