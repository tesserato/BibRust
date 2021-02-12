browserify table.js -o bundle.js -p tinyify
cargo run --release -- -i tests/bibliography.csv -o tests/bibliography.html -k -r