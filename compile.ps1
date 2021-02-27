browserify src/table.js -o src/02bundle.js -p tinyify
sass themes/coal.scss src/01table.css
# cargo run --release -- -i tests/bibliography.csv -o tests/bibliography.html