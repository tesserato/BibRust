mkdir release -Force
cp target/release/bibrust.exe release
cp 00table.html release
cp 01table.css release
cp 02bundle.js release
zip release.zip release –m –r