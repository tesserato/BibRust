browserify table.js -o 02bundle.js -p tinyify
sass themes/coal.scss 01table.css
cargo run --release -- -i tests/bibliography.csv -o tests/bibliography.html -k -r