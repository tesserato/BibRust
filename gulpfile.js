var { src, dest } = require("gulp");

var browserify = require("browserify");
// var source = require("vinyl-source-stream");
var tsify = require("tsify");
// var sourcemaps = require("gulp-sourcemaps");
// var buffer = require("vinyl-buffer");

exports.default = function() {
  return src('table.ts')
    .pipe(  
      browserify()
      .plugin(tsify, { noImplicitAny: true })
      .on('error', function (error) { console.error(error.toString()); })
      .bundle()
    )
    .pipe(dest("bundle.js"));
}


// gulp.task("default", done => { 
//   console.log('Hello World');
//   browserify()
//     .plugin(tsify, { noImplicitAny: true })
//     .bundle()
//     .on('error', function (error) { console.error(error.toString()); })
//     .pipe(dest("bundle.js"));
//   }
// )