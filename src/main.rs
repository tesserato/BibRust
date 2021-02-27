#![feature(str_split_once)]
#![allow(non_snake_case)]

use std::fs;
use std::{cmp::Ordering, vec};
use std::io::BufReader;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::collections::HashSet;
use std::str;
use std::env;
// use std::ffi::OsStr;
use std::io;
// use chrono::NaiveDate;

extern crate csv;

extern crate clap;
// use chrono::format;
use clap::{App, Arg, ArgMatches, Result};

extern crate walkdir;
use walkdir::WalkDir;

extern crate url;
use url::Url;

extern crate crossref;
use crossref::{Crossref, Work};

// extern crate nom_bibtex;
// use nom_bibtex::*;

extern crate unidecode;
use unidecode::unidecode;

extern crate serde;
use serde::{Deserialize, Serialize};

// extern crate html_minifier;
// use html_minifier::HTMLMinifier;

// extern crate minifier;
// use minifier::js::minify;

static INTERNAL_TAG_MARKER: char ='#';
static REVIEWED: &str = "#reviewed";
static MERGED: &str = "#merged";
static RETRIEVED: &str = "#retrieved";

// static SEPARATOR: &str = ",";
static INTERNAL_SEPARATOR: &str = ",";
static NAMES_SEPARATOR: &str = " and ";

#[derive(Debug, Hash, Eq, PartialEq, Clone, Deserialize, Serialize)]
struct Name{
  first_name:String,
  last_name:String
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Entry {
  Reviewed:bool,
  Type: String,
  Key: String,
  Creators: HashMap<String, Vec<Name>>,
  Tags: HashSet<String>,
  Files: HashSet<String>,
  BrokenFiles: HashSet<String>,
  Fields_Values: HashMap<String, String>,
}

impl PartialEq for Entry {
  fn eq(&self, other: &Self) -> bool {
      self.Type == other.Type &&
      self.Creators == other.Creators &&
      self.Fields_Values == other.Fields_Values &&
      self.Tags == other.Tags &&
      self.Files == other.Files
  }
}

impl Eq for Entry {}

impl PartialOrd for Entry {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Entry {
  fn cmp(&self, other: &Self) -> Ordering {
    self.Key.cmp(&other.Key)
  }
}

fn Entry_to_String_bib(e: & Entry) -> String{
  // type and key
  let mut s = format!("@{}{{{},\n",e.Type, e.Key);

  // authors
  if e.Creators.len() > 0 {
    let mut creators = e.Creators.iter().map(|x| x.0.to_string()).collect::<Vec<String>>();
    creators.sort();
    for c in creators{
      let t = names_to_string(&e.Creators[&c]);
      if !t.trim().is_empty(){
        s.push_str(&format!("{} = {{{}}},\n", c, t));
      }
    }
  }

  // Fields & Values
  if e.Fields_Values.len() > 0 {
    let mut fields = e.Fields_Values.iter().map(|x| x.0.to_string()).collect::<Vec<String>>();
    fields.sort();
    let t = fields.iter().map(|x| format!("{} = {{{}}},\n", x, e.Fields_Values[x])).collect::<Vec<String>>().join("");
    // let t = e.Fields_Values.iter().map(|x| format!("{} = {{{}}},\n", x.0, x.1)).collect::<Vec<String>>().join("");
    s.push_str(&t);
  }

  //Files
  if e.Files.len() > 0 {
    let t = hashset_to_string(&e.Files);
    s.push_str(&format!("file = {{{}}},\n", t));
  }

  //Tags
  if !e.Tags.is_empty() || e.Reviewed {
    let mut tags =e.Tags.clone();
    if e.Reviewed{
      tags.insert(REVIEWED.to_string());
    }
    let t = hashset_to_string(&tags);
    s.push_str(&format!("mendeley-tags = {{{}}},\n", t));
  }
  s.push_str("}\n");
  s
}

fn read_bib(path:PathBuf, bib_lines:&mut Vec<String>){
  let file = File::open(path).unwrap();
  let file_buffer = BufReader::new(file);

  let mut inside_comment= false;
  for line in file_buffer.lines(){
    let l = match line {
      Ok(l) => l.trim().to_string(),
      Err(e) => {
        println!("error reading line: {:?}", e);
        "".to_string()
      }
    };

    if l.is_empty(){
      continue;
    }

      if l.to_lowercase().starts_with("@comment"){
      inside_comment = true;
    }

    if l.starts_with("@") && !l.ends_with(",") && !inside_comment{
      println!("Flat entry found: {}", l);

      let mut sls = l.split("}, ");
      let (sl0, sl1) = sls.next().unwrap().split_once(",").unwrap();
      bib_lines.push(format!("{},", sl0.trim()));
      bib_lines.push(format!("{}}},", sl1.trim()));
      while let Some(sl) = sls.next(){        
        bib_lines.push(format!("{}}},", sl.trim_end().trim_end_matches("}}")));
      }
        bib_lines.push(format!("}}"));
    }

    if !inside_comment && !l.starts_with('%'){
      bib_lines.push(l.to_string().replace("“", "\"").replace("”", "\""));
    }
    if inside_comment && l.ends_with('}'){
      inside_comment = false;
    }
  }
}

fn parse_creators_field(original_value:&str) -> Vec<Name>{
  let mut authors: Vec<Name> = vec![];
  let pattern : &[_] = &[',',';','{','}','\\'];
  // let exceptions = [' ', '-'];
  fn parse(name:&str) -> String{
    let remove = [',',';','{','}','\\'];
    name
    .trim()
    .chars()
    .filter(|x|
        !x.is_ascii_control() &&
        !x.is_numeric() &&
        // !x.is_whitespace() &&
        !remove.contains(x)
    )
    .collect::<String>()
    .trim()
    .to_string()
  }
  for fl in original_value.trim_matches(pattern).split(" and ").map(|x| x.trim()){
    if fl.contains(","){
      let fl_vec = fl.split_once(",").unwrap();
      authors.push(Name{
        first_name:parse(fl_vec.1), 
        last_name:parse(fl_vec.0)
      })
    }
    else if fl.contains(" "){
      let fl_vec = fl.rsplit_once(" ").unwrap();
      authors.push(Name{
        first_name:parse(fl_vec.0),
        last_name:parse(fl_vec.1)
      })
    }
    else{
      authors.push(Name{
        first_name:"".to_string(), 
        last_name:parse(fl)
    })
    }
  }
  authors
}

fn parse_tags_field(e: &mut Entry, original_value:&str) {
  // let patterns : &[_] = &['{', '}','\t',',',' ',','];
  let mut tags: HashSet<String> = 
    original_value
      .replace(";", ",")
      .split(",").map(|x| x.to_lowercase().trim_matches(|c:char| c != INTERNAL_TAG_MARKER && !c.is_alphabetic()).to_owned())
      .filter(|s| !s.is_empty())
      .collect();
  if tags.contains(REVIEWED) {
    e.Reviewed =true;
    tags.remove(REVIEWED);
  }
  e.Tags.extend(tags);
}

fn parse_file_field(e: &mut Entry, value:&String) -> HashSet<String>{
  let patterns : &[_] = &['}',',',' '];
  let mut checked: HashSet<String> = HashSet::new();
  for raw_f in value.split(";"){
    let paths = raw_f
      .split(":")
      .filter(|x| !x.is_empty() && x.contains("."))
      .map(|x| format!("C:{}", x.trim_matches(patterns).replace("\\\\", "/").replace("\\", "/")))
      .collect::<Vec<String>>();
      for p in paths{
        if Path::new(&p).exists(){
          // println!("{}", p);
          checked.insert(Path::new(&p).as_os_str().to_str().unwrap().to_string());
        }
        else{
          e.BrokenFiles.insert(p);
        }
      }
    }
  checked
}

fn parse_url_field(value:&String) -> String{
  let pattern : &[_] = &['{', '}','\t',',',' ',';'];
  let mut checked:Vec<String> = vec![];
  for url in value.split(pattern){
    let curl = Url::parse(url);
    if curl.is_ok(){
      checked.push(curl.unwrap().to_string())
    }

  }
  checked.join(INTERNAL_SEPARATOR)
}

fn parse_edition_field(value:&String) -> String{
  let edition = value.to_lowercase();
  let mut norm_edition = String::new();

  let numeric = edition.chars().filter(|x| x.is_digit(10)).next();

  if numeric.is_some(){
    norm_edition.push(numeric.unwrap());
    return norm_edition
  }

  if edition.contains("first") {
    norm_edition="1".to_string();
  }
  else if edition.contains("second") {
    norm_edition="2".to_string();
  }
  else if edition.contains("third") {
    norm_edition="3".to_string();
  }
  else if edition.contains("fourth") {
    norm_edition="4".to_string();
  }
  else if edition.contains("fifth"){
    norm_edition="5".to_string();
  }
  else if edition.contains("sixth") {
    norm_edition="6".to_string();
  }
  else if edition.contains("seventh") {
    norm_edition="7".to_string();
  }
  else if edition.contains("eighth") {
    norm_edition="8".to_string();
  }
  else{
    norm_edition = edition;
  }
  norm_edition
}

fn parse_generic_field(original_value:&str) -> String{
  let patterns : &[_] = &['\t',',',' ','"','\'','{','}'];
  original_value
  .trim()
  .trim_matches(patterns)
  .replace("{", "")
  .replace("}", "")
  // .replacen("{", "",1)
  // .chars().rev().collect::<String>()
  // .replacen("}", "",1)
  // .chars().rev().collect::<String>()
}

fn parse_field_value(field: &str, value: &mut String, last_entry: &mut Entry){
  // let pattern : &[_] = &['\t',',',' ','"','\'','{','}'];
  match field {
    "file" => last_entry.Files = parse_file_field(last_entry, &value.to_string()),
    "author" | "editor" | "translator" => {
      let creators =parse_creators_field(&value);
      if creators.len() > 0{
      let _ = last_entry.Creators.insert(field.to_string(), creators);
      }
    },
    "mendeley-groups"|"mendeley-tags"|"groups"|"tags" => parse_tags_field( last_entry,&value),
    _ => {
      match field.as_ref() {
        "isbn" => *value = value.chars().filter(|x| x.is_numeric()).collect::<String>(),
        // "arxivid" | "eprint" => *value = value.split(":").map(|x| x.to_string()).collect::<Vec<String>>().last().unwrap().trim_matches(pattern).to_string(),
        "url" => *value = parse_url_field(&value),
        "edition" => *value = parse_edition_field(&value),
        _ => *value = parse_generic_field(&value),
      }
      if last_entry.Fields_Values.contains_key(field){
        println!("Repeated entry at {} {}\n", field, value);
      }
      else if !value.trim().is_empty(){
        last_entry.Fields_Values.insert(field.to_string(), value.clone());
      }
    }
  };
}

fn parse_bib(lines:&Vec<String> )->Vec<Entry>{
  let mut Entries : Vec<Entry> = vec![];
  let patterns : &[_] = &['{', '}','\t',',']; 
  let mut counter = 0;
  while counter < lines.len() {
    if lines[counter].trim_start().starts_with("@"){ // found entry
      let vec: Vec<&str> = lines[counter].splitn(2,"{").collect();
      if vec.len() == 2 {
        let etype = vec[0].trim().trim_matches('@').to_lowercase();
        let key = vec[1].trim().trim_matches(',').to_string();
        Entries.push(Entry{Type:etype, Key:key,..Default::default()}) ;
      }
      else{
        println!("Problem: {}\n",lines[counter]);
      }
      counter +=1;
      while counter < lines.len() && lines[counter].trim() != "}" && !lines[counter].trim_start().starts_with("@"){ // while inside entry
        let mut field_value= String::new();
        while 
        counter < lines.len() - 1 && 
        !(lines[counter].trim().ends_with("}") && lines[counter+1].trim() == "}" ) && 
        !lines[counter].trim().ends_with("},") && 
        !lines[counter+1].contains("=")
        {
          field_value.push_str(lines[counter].trim_matches('\n'));
          counter +=1;
        }
        field_value.push_str(lines[counter].trim_matches('\n'));

        let vec: Vec<&str> = field_value.splitn(2,"=").collect();
        if vec.len() == 2 {
          let field:&str = &vec[0].trim().trim_matches(patterns).to_lowercase();
          let mut value = vec[1].to_string();
          let mut last_entry = Entries.last_mut().unwrap();
          parse_field_value(field, &mut value, &mut last_entry);
        }
        else{
          println!("Split vector with size != 2 at {}\n", field_value);
        }
        counter +=1
      }
    }
    counter +=1;
  }
  Entries
}

fn names_to_string(names: &Vec<Name>) -> String{
  names
  .iter()
  .map(|a| a.first_name.to_owned() + " " + &a.last_name)
  .collect::<Vec<String>>()
  .join(NAMES_SEPARATOR)
}

fn hashset_to_string(hs: &HashSet<String>) -> String{
  hs.to_owned().into_iter().collect::<Vec<String>>().join(INTERNAL_SEPARATOR)
}

fn write_csv(path: &PathBuf, entries: &Vec<Entry>, ordered_fields: &Vec<String>){
  // let path = Path::new(path);
  let display = path.display();

  // Open a file in write-only mode, returns `io::Result<File>`
  let mut f = match File::create(&path) {
      Err(why) => panic!("couldn't create {}: {}", display, why),
      Ok(file) => file,
  };

  write!(f,"\u{feff}").expect("Couldn't write BOM while writing .csv"); // BOM, indicating uft8 for excel

  let top_row
  = String::from("reviewed,entry type,key,author,editor,translator,") + (&ordered_fields.join(",")) + ",tags,file";
  writeln!(f, "{}", top_row).unwrap();

  for e in entries{
    let c0:String = match e.Reviewed{
      true => "x".to_string(),
      false => "".to_string(),
    };

    let ca:String = match e.Creators.contains_key("author"){
      true => names_to_string(&e.Creators["author"]),
      false => "".to_string(),
    };

    let ce:String = match e.Creators.contains_key("editor"){
      true => names_to_string(&e.Creators["editor"]),
      false => "".to_string(),
    };
    
    let ct:String = match e.Creators.contains_key("translator"){
      true => names_to_string(&e.Creators["translator"]),
      false => "".to_string(),
    };
    
    let mut row:Vec<String> = vec![c0, e.Type.to_owned(), e.Key.to_owned(), ca, ce, ct];

    for field in ordered_fields{
      if e.Fields_Values.contains_key(field){
        row.push(
          format!("\"{}\"", e.Fields_Values[field].to_owned().replace("\"", "\"\""))
        );
      }
      else{
        row.push(" ".to_string());
      }
    }
    row.push(format!("\"{}\"", hashset_to_string(&e.Tags)));
    row.push(format!("\"{}\"", hashset_to_string(&e.Files)));
    writeln!(f, "{}",row.join(INTERNAL_SEPARATOR)).unwrap();
  }
}

fn write_html(path: &PathBuf, entries: &Vec<Entry>){
  // let mut html = std::fs::read_to_string("00table.html").expect("Something went wrong reading table.html");
  // let css = std::fs::read_to_string("01table.css").expect("Something went wrong reading 01table.css");
//   let js = std::fs::read_to_string("02bundle.js").expect("Something went wrong reading 02tabulator.js");

  let mut html = include_str!("00table.html").to_string();
  let css = include_str!("01table.css").to_string();
  let js = include_str!("02bundle.js").to_string();


  // let mut js = tabulator;
  // js.push_str(&entries_to_js_obj(entries));
  // js.push_str(&table);
  // js = minify(&js);

  html = html
    .replace(
    "  <link href=\"01table.css\" rel=\"stylesheet\">",
     &format!("<style>\n{}\n</style>", css)
    )
    // .replace(
    //   "  <script type=\"text/javascript\" src=\"02tabulator.js\"></script>",
    //   "")
    .replace(
      "  <script type=\"text/javascript\" src=\"tabledata.js\"></script>",
      ""
    )
    .replace(
      "  <script src=\"02bundle.js\"></script>",
      &format!("<script>\n{}\n{}\n</script>", &entries_to_js_obj(entries), js)
    );
    

  let display = path.display();

  let mut f = match File::create(&path) {
      Err(why) => panic!("couldn't create {}: {}", display, why),
      Ok(file) => file,
  };

  // let mut html_minifier = HTMLMinifier::new();
  // match html_minifier.digest(html) {
  //   Ok(_)  => (),//println!("{:?}", m),
  //   Err(e) => println!("{:?}", e),
  // }
  // f.write_all(html_minifier.get_html()).expect("unable to write html to disk");

  f.write_all(html.as_bytes()).expect("unable to write html to disk");
}

fn entries_to_js_obj(entries: &Vec<Entry>) -> String{
  fn remove_backticks(text: &String) -> String{
    text.chars().filter(|x| x != &'`').collect::<String>().replace("\\", "/").replace("$", "\\$")
  }
  let mut obj = "tabledata = [\n".to_string();
  for e in entries{
      // type and key
  let mut row: Vec<String> = vec![];

  if e.Reviewed{
    row.push(format!("reviewed: true"));
  }

  row.push(format!("key: `{}`", e.Key));
  row.push(format!("type: `{}`", e.Type));

  // authors
  if e.Creators.len() > 0 {
    for (c, v) in &e.Creators{
      let t = names_to_string(v);
      row.push(format!("{}: `{}`", c, remove_backticks(&t)));
    }
  }

  // Fields & Values
  if e.Fields_Values.len() > 0 {
    let mut fields = e.Fields_Values.iter().map(|x| x.0.to_string()).collect::<Vec<String>>();
    fields.sort();
    let t = fields.iter().map(|x| format!("{}: `{}`", x, remove_backticks(&e.Fields_Values[x]))).collect::<Vec<String>>().join(",");
    // let t = e.Fields_Values.iter().map(|x| format!("{} = {{{}}},\n", x.0, x.1)).collect::<Vec<String>>().join("");
    row.push(t);
  }

  //Files
  if e.Files.len() > 0 {
    let t = hashset_to_string(&e.Files);
    row.push(format!("file: `{}`", remove_backticks(&t)));
  }

  //Tags
  if !e.Tags.is_empty() {
    let t = hashset_to_string(&e.Tags);
    row.push(format!("tags: `{}`", remove_backticks(&t)));
  }
  obj.push_str(&format!("  {{{}}},\n",row.join(",")));
  }
  obj.push_str(&format!("];\n"));
  obj
}

fn write_js_object(path: &PathBuf, entries: &Vec<Entry>){
  let mut f = match File::create(&path) {
    Err(why) => panic!("couldn't create {}: {}", path.display(), why),
    Ok(file) => file,
  };
  writeln!(f, "{}", entries_to_js_obj(entries)).unwrap();
}

fn write_json(path: &PathBuf, entries: &Vec<Entry>){
  let mut f = match File::create(&path) {
    Err(why) => panic!("couldn't create {}: {}", path.display(), why),
    Ok(file) => file,
  };

  writeln!(f, "{}", serde_json::to_string(entries).unwrap()).unwrap();

}

fn write_bib(path: &PathBuf, entries: & Vec<Entry>){
  // let path = Path::new(path);
  let display = path.display();

  // Open a file in write-only mode, returns `io::Result<File>`
  let mut f = match File::create(&path) {
      Err(why) => panic!("couldn't create {}: {}", display, why),
      Ok(file) => file,
  };

  for e in entries{
    writeln!(f, "{}", Entry_to_String_bib(e)).unwrap();
  }
}

fn find_paths_to_files_with_ext(root_path:&PathBuf, exts:& Vec<String> ) -> Vec<PathBuf>{
  let mut paths: Vec<PathBuf> = vec![];
  for direntry in WalkDir::new(root_path).into_iter().filter_map(|e| e.ok()){
    let ext = direntry.path().extension();
    if ext.is_some() && exts.contains(&ext.unwrap().to_str().unwrap().to_lowercase()){
      paths.push(direntry.path().to_owned());
    }
  }
  paths
}

fn get_entries_from_root_path(root_path:PathBuf) -> Vec<Entry>{
  let exts=vec!["bib".to_string()];
  let bib_paths = find_paths_to_files_with_ext(&root_path, &exts);

  let mut bib_vec = vec![];
  for p in bib_paths {
    println!("{:#?}", p);
    read_bib(p, &mut bib_vec);
  }
  // write_raw_bib("Complete_raw.bib", &mut bib_vec);

  parse_bib(&bib_vec)
}

fn read_and_parse_csv(path:PathBuf) -> Vec<Entry>{
  let mut rdr = csv::ReaderBuilder::new().from_path(path).unwrap();

  let keys:Vec<String> = rdr.headers().unwrap().into_iter().map(|x| x.to_string()).collect();
  // let n = keys.len();
  println!("Found the folowing fields in .csv header: {}", keys.join(" | "));
  let mut Entries : Vec<Entry> = vec![];

  while let Some(result) = rdr.records().next() {
    match result {
      Ok(_) => (),
      Err(e) =>{
        println!("{:?}\n", e);
        continue
      },
    }
    let mut v:Vec<String> = result.unwrap().into_iter().map(|x| x.to_string()).collect();
    let mut e = Entry{Type: v[1].to_lowercase().to_owned(), Key: v[2].to_owned(),..Default::default()};

    if !v[0].is_empty(){
      e.Reviewed=true;
    }
    for i in 3..keys.len() {
      parse_field_value(&keys[i].to_lowercase(), &mut v[i],&mut e);
    }
    Entries.push(e);
  }
  Entries
}

fn parse_cl_args() -> ArgMatches<'static> {
  App::new("SAMARA")
    .version("1.0")
    .about("Tame your (bib)liography!")
    .author("https://carlos-tarjano.web.app/")
    .arg(Arg::with_name("input_pos")
      // .short("ip")
      .long("INPUT")
      .value_name("INPUT")
      .help("positional alternative to -i: The path to the input file to convert from, or the root folder in which to search for .bib files")
      .takes_value(true)
      .required(false)
      .index(1)
      .conflicts_with("input")
    )
    .arg(Arg::with_name("output_pos")
      // .short("op")
      .long("OUTPUT")
      .value_name("OUTPUT")
      .help("positional alternative to -o: Path to output file")
      .takes_value(true)
      .required(false)
      .index(2)
      .conflicts_with("output")
    )
    .arg(Arg::with_name("input")
      .short("i")
      .long("input")
      .value_name("path to file or folder")
      .help("The path to the input file to convert from, or the root folder in which to search for .bib files")
      .takes_value(true)
      .required(false)
    )
    .arg(Arg::with_name("output")
      .short("o")
      .long("output")
      .value_name("path to file")
      .help("Path[s] to output file")
      .takes_value(true)
      .required(false)
    )
    .arg(Arg::with_name("auxiliary")
      .short("a")
      .long("auxiliary")
      .value_name("path to file or folder")
      .help("The file, or the root folder in which to search for .bib files to be used to complement the info")
      .takes_value(true)
      .required(false)
    )
    .arg(Arg::with_name("files")
      .short("f")
      .long("files")
      .value_name("path to folder")
      .help("Path to a folder with files (.pdf, .epub and / or .djvu) to be linked to entries")
      .takes_value(true)
      .required(false)
    )
    .arg(Arg::with_name("clean")
      .short("c")
      .long("clean")
      .value_name("clean entries?")
      .help("if set, new unique keys will be created, and other cleaning procedures will be executed")
      .takes_value(false)
      .required(false)
    )
    .arg(Arg::with_name("lookup")
      .short("l")
      .long("lookup")
      .value_name("lookup entries?")
      .help("if set, searches for data from crossref for entries with doi and prompts the user to accept changes. An e-mail is needed as argument")
      .takes_value(true)
      .required(false)
  )
    .arg(Arg::with_name("merge")
      .short("m")
      .long("merge")
      .value_name("merge redundant entries?")
      .help("if set, redundant entries will be merged in a nondestructive way (except for keys and type)")
      .takes_value(false)
      .required(false)
    )
    .arg(Arg::with_name("rename")
      .short("r")
      .long("rename")
      .value_name("rename files?")
      .help("if set, files pointed in the entries will be renamed")
      .takes_value(false)
      .required(false)
    )
    .get_matches()
}

fn from_path_to_entries(input_path: String) -> Option<Vec<Entry>>{
  let mut main_entries = vec![];
  let p = PathBuf::from(input_path);
  if p.is_dir(){
    println!("Searching for .bib files in {:#?}:", p);
    main_entries = get_entries_from_root_path(p);
  }
  else if p.is_file() && p.extension().is_some(){
    match p.extension().unwrap().to_str() {
      Some("csv")    => main_entries = read_and_parse_csv(p),
      Some("bib")    => {
        let mut bib_vec = vec![];
        read_bib(p, &mut bib_vec);
        main_entries = parse_bib(&bib_vec);
      },
      Some(ext) => println!("Couldn't recognize extension .{}", ext),
      None           => println!("No extension detected in {:?}", p.as_os_str()),
    }
  }
  else{
    return None;
    // panic!("No extension detected in {:?}; no input is available", p.as_os_str());
    // panic!("Oh no something bad has happened!")
  }
  Some(main_entries)
}
#[derive(Default)]
struct Statistics{
  types: HashMap<String, u32>,
  fields: HashMap<String, u32>,
  keys: HashSet<String>,
  creators: HashSet<Name>,
  has_doi:usize,
  has_file:usize,
  has_url:usize,
  has_author:usize,
  reviewed:usize,
  ordered_fields:Vec<String>,
  ordered_types:Vec<String>
}

fn generate_key(entry: &Entry) -> String{
  let mut year = "year".to_string();
  if entry.Fields_Values.contains_key("year") && entry.Fields_Values["year"].len() >= 2 {
    year = entry.Fields_Values["year"].to_owned();
  }
  else if entry.Fields_Values.contains_key("date"){
    let mut chars = entry.Fields_Values["date"].chars().filter(|x| x.is_digit(10));
    let mut optyear = String::new();
    let mut char = chars.next();
    while optyear.len() < 4 && char.is_some(){
      optyear.push(char.unwrap());
      char = chars.next();
    }
    if optyear.trim().is_empty() {
      year = "year".to_string();
    }
    else{
      year = optyear;
    }
  }

  let mut roles: Vec<String> = vec![];
  for (c, names) in &entry.Creators{
    if !names.is_empty(){
      roles.push(c.to_owned());
    }
  }
  roles.sort();
  let mut creator = "creator".to_string();
  let mut etal = "";
  'outer:for role in roles{
    for name in &entry.Creators[&role]{
      if !name.last_name.trim().is_empty(){
        creator = unidecode(&name.last_name);
        if entry.Creators[&role].len() > 1 {
          etal = "_etal";
        }
        break 'outer;
      }
    }
  }

  format!("{}_{}{}", year, creator.chars().filter(|x| x.is_alphabetic()).collect::<String>().to_lowercase(), etal)
}

fn get_statistics_and_clean(Entries:&mut Vec<Entry>, clean:bool) -> Statistics{
  if clean{
    println!("Cleaning entries");
  }
  let mut stats = Statistics{..Default::default()};
  let n = Entries.len();
  for entry in Entries{
    if clean{
      // generating unique key
      let base_key = generate_key(entry);
      let mut ctr = 0;
      let mut key = base_key.clone();
      while stats.keys.contains(&key) {
        ctr +=1;
        key = format!{"{}{}", base_key, ctr};
      }      
      entry.Key = key;


      // for file in entry.Files.into() {

      // }

      // date and year
      let mut date: Vec<String> = vec![];
      if entry.Fields_Values.contains_key("date"){
        date = entry.Fields_Values["date"].split("-").map(|x| x.to_owned()).collect::<Vec<String>>();
        if entry.Fields_Values.contains_key("year"){
          if date[0] == entry.Fields_Values["year"]{
           entry.Fields_Values.remove("date");
           entry.Fields_Values.remove("month");
          }
        }
        else{
          entry.Fields_Values.insert("year".to_string(), date[0].clone());
          entry.Fields_Values.remove("date");
          entry.Fields_Values.remove("month");
        }
      }
      // journaltitle
      if entry.Fields_Values.contains_key("journaltitle"){
        if entry.Fields_Values.contains_key("journal"){
          if entry.Fields_Values["journal"] == entry.Fields_Values["journaltitle"]{
            entry.Fields_Values.remove("journaltitle");
          }
        }
        else{
          entry.Fields_Values.insert("journal".to_string(), entry.Fields_Values["journaltitle"].clone());
          entry.Fields_Values.remove("journaltitle");
        }
      }
      // booktitle
      if entry.Fields_Values.contains_key("booktitle"){
        if entry.Fields_Values.contains_key("journal"){
          if entry.Fields_Values["journal"] == entry.Fields_Values["booktitle"]{
            entry.Fields_Values.remove("booktitle");
          }
        }
        else{
          entry.Fields_Values.insert("journal".to_string(), entry.Fields_Values["booktitle"].clone());
          entry.Fields_Values.remove("booktitle");
        }
      }
      // eventtitle
      if entry.Fields_Values.contains_key("eventtitle"){
        if entry.Fields_Values.contains_key("journal"){
          if entry.Fields_Values["journal"] == entry.Fields_Values["eventtitle"]{
            entry.Fields_Values.remove("eventtitle");
          }
        }
        else{
          entry.Fields_Values.insert("journal".to_string(), entry.Fields_Values["eventtitle"].clone());
          entry.Fields_Values.remove("eventtitle");
        }
      }
      // shorttitle
      if entry.Fields_Values.contains_key("shorttitle"){
        if entry.Fields_Values.contains_key("title"){
          if entry.Fields_Values["title"] == entry.Fields_Values["shorttitle"]{
            entry.Fields_Values.remove("shorttitle");
          }
        }
        else{
          entry.Fields_Values.insert("title".to_string(), entry.Fields_Values["shorttitle"].clone());
          entry.Fields_Values.remove("shorttitle");
        }
      }
    }
    stats.keys.insert(entry.Key.clone());

    for (_, creators) in &entry.Creators{
      for creator in creators{
        stats.creators.insert(creator.to_owned());
      }
    }

    if entry.Reviewed{
      stats.reviewed += 1;
    }

    if stats.types.contains_key(&entry.Type){
      *stats.types.get_mut(&entry.Type).unwrap() += 1;
    }else{
      stats.types.insert(entry.Type.to_string(), 1);
    }

    if entry.Files.len() > 0{
      stats.has_file += 1;
    }

    if entry.Creators.len() > 0{
      stats.has_author += 1;
    }
  
    for (field, value) in &entry.Fields_Values{
      if value.trim().is_empty(){
        continue
      }
      match field.as_ref() {
        "doi" => stats.has_doi += 1,
        "url" => stats.has_url += 1,
        _ => (),
      }
      if stats.fields.contains_key(field){
        *stats.fields.get_mut(field).unwrap() += 1;
      }else{
        stats.fields.insert(field.to_string(), 1);
      }
    }
  }
  
  // Sorting
  let mut types_vec: Vec<(String, u32)> = vec![];
  for (t, c) in &stats.types{
    types_vec.push((t.to_string(), *c));
  }  

  println!(
    "\nFound a total of {} entries\n({} reviewed, {} with author, {} with doi, {} with files, {} whith url):", 
                        n, stats.reviewed, stats.has_author, stats.has_doi, stats.has_file, stats.has_url);
    
  types_vec.sort();
  types_vec.sort_by(|a, b| a.1.cmp(&b.1).reverse());
  for (key, value) in types_vec {
    println!("{} {}", key, value);
    stats.ordered_types.push(key);
  }

  println!("\nFields:");
  let mut fields_vec: Vec<(String, u32)> = vec![];

  for (t, c) in &stats.fields{
    fields_vec.push((t.to_string(), *c));
  }  

  fields_vec.sort();
  fields_vec.sort_by(|a, b| a.1.cmp(&b.1).reverse());
  for (key, value) in fields_vec {
    println!("{} {}", key, value);
    stats.ordered_fields.push(key);
  }
  println!("");
  stats
}

fn parse_crossref(w:Work, e: &mut Entry) -> bool{
  println!("Processing entry:");

  let new_type = match w.type_.as_ref() {
    "journal-article" => "article".to_string(),
    "proceedings-article" => "inproceedings".to_string(),
    "book" | "reference-book" | "edited-book" => "book".to_string(),
    "book-chapter" | "book-section" | "book-part" | "book-track" => "inbook".to_string(),
    "dissertation" | "monograph" => "thesis".to_string(),
    _ =>  "misc".to_string(),
  };

// {"id":"book-section","label":"Book Section"},
// {"id":"book-part","label":"Part"}
// {"id":"book-track","label":"Book Track"}
// {"id":"journal-article","label":"Journal Article"}
// ,{"id":"book","label":"Book"},
// {"id":"proceedings-article","label":"Proceedings Article"}
// {"id":"reference-book","label":"Reference Book"}
// {"id":"book-chapter","label":"Book Chapter"}
// {"id":"edited-book","label":"Edited Book"}
// {"id":"dissertation","label":"Dissertation"}
// {"id":"monograph","label":"Monograph"}

// ,{"id":"report","label":"Report"},{"id":"peer-review","label":"Peer Review"},,,,{"id":"other","label":"Other"}{"id":"journal-volume","label":"Journal Volume"},{"id":"book-set","label":"Book Set"},{"id":"reference-entry","label":"Reference Entry"},,{"id":"journal","label":"Journal"},{"id":"component","label":"Component"},,{"id":"proceedings-series","label":"Proceedings Series"},{"id":"report-series","label":"Report Series"},{"id":"proceedings","label":"Proceedings"},{"id":"standard","label":"Standard"},,{"id":"posted-content","label":"Posted Content"},{"id":"journal-issue","label":"Journal Issue"},,{"id":"dataset","label":"Dataset"},{"id":"book-series","label":"Book Series"},,{"id":"standard-series","label":"Standard Series"}]}}

  let mut new_entry = Entry{Type: new_type, Key: e.Key.clone(),..Default::default()};

  match e.Fields_Values.get("doi") {
    Some(d) => {new_entry.Fields_Values.insert("doi".to_string(), d.to_owned()); ()},
    None => println!("Doi error"),
  };

  if e.Files.len() > 0 {
    println!("file(s): {}", hashset_to_string(&e.Files));
    new_entry.Files = e.Files.clone();
  }
  println!("type: {} → {}", e.Type, new_entry.Type);

  new_entry.Tags = e.Tags.clone();
  new_entry.Tags.insert(RETRIEVED.to_string());

  new_entry.Fields_Values.insert("title".to_string(), w.title[0].clone());
  let val = match e.Fields_Values.get("title"){
    Some(v) => v,
    None => "",
  };
  println!("title: {} → {}", val, new_entry.Fields_Values["title"]);

  match w.issued.date_parts.0[0][0]{/////////////////////////////////////////////////////
    Some(year) => {
    new_entry.Fields_Values.insert("year".to_string(), year.to_string());
    let val = match e.Fields_Values.get("year"){
      Some(v) => v,
      None => "",
    };
    println!("year: {} → {}", val, new_entry.Fields_Values["year"]);
  },
    None => (),
  }
  match w.author{/////////////////////////////////////////////////////
    Some(author) => {
      let mut fauthors:Vec<Name> = vec![];
      for a in author{
        let first = match a.given {
          Some(n) => n,
          None => "".to_string(),
        };
        fauthors.push(Name{first_name:first, last_name:a.family});
      }
      new_entry.Creators.insert("author".to_string(), fauthors);
      let val = match e.Creators.get("author"){
        Some(v) => names_to_string(v),
        None => "".to_string(),
      };
      println!("author: {} → {}", val, names_to_string(new_entry.Creators.get("author").unwrap()));
    }
    None => (),
  }
  match w.editor{/////////////////////////////////////////////////////
    Some(author) => {
      let mut fauthors:Vec<Name> = vec![];
      for a in author{
        let first = match a.given {
          Some(n) => n,
          None => "".to_string(),
        };
        fauthors.push(Name{first_name:first, last_name:a.family});
      }
      new_entry.Creators.insert("editor".to_string(), fauthors);
      let val = match e.Creators.get("editor"){
        Some(v) => names_to_string(v),
        None => "".to_string(),
      };
      println!("editor: {} → {}", val, names_to_string(new_entry.Creators.get("editor").unwrap()));
    }
    None => (),
  }
  match w.translator{/////////////////////////////////////////////////////
    Some(author) => {
      let mut fauthors:Vec<Name> = vec![];
      for a in author{
        let first = match a.given {
          Some(n) => n,
          None => "".to_string(),
        };
        fauthors.push(Name{first_name:first, last_name:a.family});
      }
      new_entry.Creators.insert("translator".to_string(), fauthors);
      let val = match e.Creators.get("translator"){
        Some(v) => names_to_string(v),
        None => "".to_string(),
      };
      println!("translator: {} → {}", val, names_to_string(new_entry.Creators.get("translator").unwrap()));
    }
    None => (),
  }
  match w.container_title{/////////////////////////////////////////////////////
    Some(journal) => {
      let jn = match journal.get(0) {
        Some(j) => j,
        None => "",
      };
      new_entry.Fields_Values.insert("journal".to_string(), jn.to_string());
      let val = match e.Fields_Values.get("journal"){
        Some(v) => v,
        None => "",
      };
      println!("journal: {} → {}", val, new_entry.Fields_Values["journal"]);
    },
    None => (),
  }
  match w.link{/////////////////////////////////////////////////////
    Some(link) => {
      let lk = match link.get(0) {
        Some(l) => l.url.clone(),
        None => "".to_string(),
      };
      new_entry.Fields_Values.insert("url".to_string(), lk);
      let val = match e.Fields_Values.get("url"){
        Some(v) => v,
        None => "",
      };
      println!("url: {} → {}", val, new_entry.Fields_Values["url"]);
    },
    None => (),
  }
  match w.isbn{/////////////////////////////////////////////////////
    Some(isbn) => {
      new_entry.Fields_Values.insert("isbn".to_string(), isbn.join(","));
      let val = match e.Fields_Values.get("isbn"){
        Some(v) => v,
        None => "",
      };
      println!("isbn: {} → {}", val, new_entry.Fields_Values["isbn"]);
    },
    None => (),
  }
  match w.issn{/////////////////////////////////////////////////////
    Some(issn) => {
      new_entry.Fields_Values.insert("issn".to_string(), issn.join(","));
      let val = match e.Fields_Values.get("issn"){
        Some(v) => v,
        None => "",
      };
      println!("issn: {} → {}", val, new_entry.Fields_Values["issn"]);
    },
    None => (),
  }

  new_entry.Fields_Values.insert("publisher".to_string(), w.publisher);
  let val = match e.Fields_Values.get("publisher"){
    Some(v) => v,
    None => "",
  };
  println!("publisher: {} → {}", val, new_entry.Fields_Values["publisher"]);

  match w.volume{/////////////////////////////////////////////////////
    Some(volume) => {
      new_entry.Fields_Values.insert("volume".to_string(), volume);
      let val = match e.Fields_Values.get("volume"){
        Some(v) => v,
        None => "",
      };
      println!("volume: {} → {}", val, new_entry.Fields_Values["volume"]);
    },
    None => (),
  }
  match w.page{/////////////////////////////////////////////////////
    Some(pages) => {
      new_entry.Fields_Values.insert("pages".to_string(), pages.replace("-", "--"));
      let val = match e.Fields_Values.get("pages"){
        Some(v) => v,
        None => "",
      };
      println!("pages: {} → {}", val, new_entry.Fields_Values["pages"]);
    },
    None => (),
  }
  match w.language{/////////////////////////////////////////////////////
    Some(langid) => {
      new_entry.Fields_Values.insert("langid".to_string(), langid);
      let val = match e.Fields_Values.get("langid"){
        Some(v) => v,
        None => "",
      };
      println!("langid: {} → {}", val, new_entry.Fields_Values["langid"]);
    },
    None => (),
  }
  match w.journal_issue{/////////////////////////////////////////////////////
    Some(issue) => {
      let number = issue.issue.unwrap();
      new_entry.Fields_Values.insert("number".to_string(), number);
      let val = match e.Fields_Values.get("number"){
        Some(v) => v,
        None => "",
      };
      println!("number: {} → {}", val, new_entry.Fields_Values["number"]);
    },
    None => (),
  }
  // match w.archive{/////////////////////////////////////////////////////
  //   Some(archive) => {
  //     println!("archive = {}", archive.join(","));
  //   }
  //   None => (),
  // }
  match w.subject{/////////////////////////////////////////////////////
    Some(keywords) => {
      new_entry.Fields_Values.insert("keywords".to_string(), keywords.join(",").replace(" and ", ","));
      let val = match e.Fields_Values.get("keywords"){
        Some(v) => v,
        None => "",
      };
      println!("keywords: {} → {}", val, new_entry.Fields_Values["keywords"]);
    },
    None => (),
  }
  match w.abstract_{/////////////////////////////////////////////////////
    Some(abstr) => {
      new_entry.Fields_Values.insert("abstract".to_string(), abstr);
      let val = match e.Fields_Values.get("abstract"){
        Some(v) => v,
        None => "",
      };
      println!("abstract: {} → {}", val, new_entry.Fields_Values["abstract"]);
    },
    None => (),
  }

  let mut input = String::new();
  let stdin = io::stdin(); // We get `Stdin` here.
  let mut res = stdin.read_line(&mut input);
  while res.is_err(){
    let er = res.err().unwrap();
    println!{"{}", er};
    res = stdin.read_line(&mut input);
  }
  // println!("{:?}", input);
  let res = match input.as_ref(){
    "e\r\n" => {
      println!("Exiting...");
      true
    },
    "\r\n"  => {
      *e = new_entry;
      false
    },
    _       => {
      println!("Skipping entry\n");
      false
    },
  };
 res
}

fn lookup(Entries:&mut Vec<Entry>, email: &String) {
  let client = Crossref::builder()
    .polite(email)
    .build().expect("Couldn't build client");

  println!("\nPress 'enter' to accept, enter any other input to skip current entry and 'e' to abort. using {}\n", email);
  'outer:for e in Entries{
    if e.Fields_Values.contains_key("doi") && !e.Tags.contains(RETRIEVED) && !e.Reviewed{
      let res = match client.work(&e.Fields_Values["doi"]){
        Ok(w) => parse_crossref(w, e),
        Err(er) => {println!("\n{}\n", er); false},
      };
      if res {
        break 'outer;
      }
    }
  }
}

fn temp_clean(Entries:&mut Vec<Entry>){
  for e in Entries{
    if e.Fields_Values.contains_key("broken-files"){
      e.Fields_Values.remove("broken-files");
    }
    if e.Tags.contains("#no author"){
      e.Tags.remove("#no author");
    }
  }
}

fn rename_files(Entries:&mut Vec<Entry>){
  for e in Entries{
    if !e.Files.is_empty() && e.Reviewed{
      let mut newnames:HashSet<String> = HashSet::new();
      let mut oldnames:HashSet<String> = HashSet::new();
      let typ = match e.Type.as_ref() {
        "article" | "inproceedings" | "incollection" => "a",
        "preprint" => "p",
        "book" | "collection" | "thesis" | "mvbook" | "phdthesis" => "b",
        _ => "r"
      };
      let tit = match e.Fields_Values["title"].split(":").next(){
        Some(t) => t.chars().filter(|x| x.is_alphanumeric() || x == &' ' || x == &'-' || x == &'_').collect::<String>(),
        None => "NO_TITLE".to_string()
      };
      let paths = e.Files.iter().map(|x| Path::new(x));

      for path in paths{
        if path.extension().is_some(){
          let rt = path.parent().unwrap().to_str().unwrap();
          let newname = format!("{}/!{} {{{}}} {}.{}", rt, typ, e.Key, tit, path.extension().unwrap().to_str().unwrap().trim()).replace("\\", "/");
          // let newpath = Path::new(&newname);
          // println!("\nold: {:?}", path);
          // println!("new: {:?}", newpath);
          if path.to_str().unwrap() != newname {
            match fs::rename(path, &newname){
              Ok(_) => {
                println!("\nold: {}", path.to_str().unwrap());
                println!("new: {}", newname);
                newnames.insert(newname);
                oldnames.insert(path.to_str().unwrap().to_string());
              }
              Err(e) => println!("{}", e)
            }
          }
        }
      }
      for name in oldnames{
        e.Files.remove(&name);
      }
      for name in newnames{
        e.Files.insert(name);
      }
    }
  }
}

fn relink_files_from_paths(entries: &mut Vec<Entry>, doc_paths: &Vec<PathBuf>){
  for e in entries{
    for p in doc_paths{
      let path = p.to_str().unwrap().to_owned().trim().to_string().replace("\\", "/");
      if path.contains(&format!("{{{}}}", e.Key)) && !e.Files.contains(&path){
        println!("Relinked: {}", path);
        e.Files.insert(path);
      }
    }
  }
}

fn main() -> Result<()> {
  println!("Running from {}", env::current_dir().unwrap().to_str().unwrap().to_string());
  let args = parse_cl_args();

  // read input path from args
  let mut input_path = String::new();
  if args.value_of("input").is_some(){
    input_path = args.value_of("input").unwrap().to_string();
    println!("Input: {} (from named arg)", input_path);
  }
  else if args.value_of("input_pos").is_some() {
    input_path = args.value_of("input_pos").unwrap().to_string();
    println!("Input: {} (from positional arg)", input_path);
  }
  else{
    input_path = env::current_dir().unwrap().to_str().unwrap().to_string();
    // println!("Defaulting to searching for .bib files in {:?}", input_path);
  }


  // read into main entries
  let mut main_entries = match from_path_to_entries(input_path){
    Some(me) => me,
    None => panic!("No extension detected in input path; no input is available"),
  };


  // auxiliary entries
  if args.value_of("auxiliary").is_some(){
    let aux_path = args.value_of("auxiliary").unwrap().to_string();
    let auxiliary_entries = from_path_to_entries(aux_path);
    if auxiliary_entries.is_some(){
      get_files_from_entries(&mut main_entries, &auxiliary_entries.unwrap());
    }
  }


  // file paths to relink files
  if args.value_of("files").is_some(){
    let files_root_path = PathBuf::from(args.value_of("files").unwrap().to_string());
    let exts = vec!["pdf".to_string(), "epub".to_string(), "djvu".to_string()];
    let filepaths = find_paths_to_files_with_ext(&files_root_path, &exts);
    if !filepaths.is_empty() {
      relink_files_from_paths(&mut main_entries, &filepaths);
    }
  }

  // crossref lookup
  if args.value_of("lookup").is_some(){
    let email = args.value_of("lookup").unwrap().to_string();
    lookup(&mut main_entries, &email);
  }

  // merge
  if args.is_present("merge"){
    remove_redundant_Entries(&mut main_entries);
  }

  // clean
  let mut generate_keys = false;
  if args.is_present("clean"){
    generate_keys = true;
  }
  let stats = get_statistics_and_clean(&mut main_entries, generate_keys);

  temp_clean(&mut main_entries);

  // rename associated files
  if args.is_present("rename"){
    rename_files(&mut main_entries);
  }

  // output
  let mut output_path = String::new();
  if args.value_of("output").is_some(){
    output_path = args.value_of("output").unwrap().to_string();
  }
  else if args.value_of("output_pos").is_some() {
    output_path = args.value_of("output_pos").unwrap().to_string();
  }
  else{
    return Ok(())
  }

  main_entries.sort_by(|a, b| b.Key.cmp(&a.Key));
  let mut p = PathBuf::from(output_path);

  if p.is_dir(){
    p.push("Result.html");
    println!("Saving results at {:#?}:", p);
    write_html(&p, &main_entries)
  }
  else if p.extension().is_some(){
    match p.extension().unwrap().to_str() {
      Some("csv")    => write_csv(&p, &main_entries, &stats.ordered_fields),
      Some("bib")    => write_bib(&p, &main_entries),
      Some("html")   => write_html(&p, &main_entries),
      Some("json")   => write_json(&p, &main_entries),
      Some("js")     => write_js_object(&p, &main_entries),
      Some(ext) => {
        println!("Couldn't recognise extension {}. Defaulting to .html mode.", ext);
        p.set_extension("html");
        write_html(&p, &main_entries);
      },
      None           => (), // can't be none. this code is dead.
    }
  }
  else{
    p = env::current_dir().unwrap();
    p.push("Result.html");
    write_html(&p, &main_entries);
    println!("No extension found. Defaulting to .html mode.");
  }
  return Ok(())
}

fn get_files_from_entries(entries: &mut Vec<Entry>, other_entries: &Vec<Entry>){
  for e0 in entries{
    for e1 in other_entries.into_iter().filter(|x| !x.Files.is_empty()){
      for key in vec!["title", "doi", "url", "abstract", "eprint"]{
        if
        e0.Fields_Values.contains_key(key) &&
        e1.Fields_Values.contains_key(key) &&
        e0.Fields_Values[key].to_lowercase() == e1.Fields_Values[key].to_lowercase()
        {
          e0.Files.extend(e1.Files.clone());
          break;
        }  
      }
    }
  }
}

fn remove_redundant_Entries(entries: & mut Vec<Entry>){
  // Remove identical entries
  let mut repeated: Vec<usize> = vec![];
  for i in 0..entries.len(){
    for j in (i+1)..entries.len(){
      if 
      entries[i].Creators == entries[j].Creators &&
      entries[i].Fields_Values == entries[j].Fields_Values &&
      entries[i].Type == entries[j].Type
      { 
        if entries[j].Reviewed{
          entries[i].Reviewed = true;
        }
        entries[i].Files = entries[i].Files.union(&entries[j].Files).map(|x| x.to_owned()).collect();
        entries[i].Tags = entries[i].Tags.union(&entries[j].Tags).map(|x| x.to_owned()).collect();
        repeated.push(j);
      }
    }
  }  
  println!("Removed {} identical entries", &repeated.len());
  repeated.sort_unstable_by(|a, b| b.cmp(a));
  repeated.dedup();

  for i in repeated{
    entries.remove(i);
  }

  // Remove identical entries
  let mut repeated: Vec<usize> = vec![];
  for i in 0..entries.len(){
    if entries[i].Fields_Values.contains_key("title") && entries[i].Reviewed {
      for j in (i+1)..entries.len(){
        if entries[j].Fields_Values.contains_key("title"){
          let val_i: String = entries[i].Fields_Values["title"].trim().to_lowercase().chars().filter(|x| x.is_alphanumeric()).collect();
          let val_j: String = entries[j].Fields_Values["title"].trim().to_lowercase().chars().filter(|x| x.is_alphanumeric()).collect();
          if val_i == val_j{
            // println!("\n{}", entries[i].Fields_Values["title"]);
            // println!("{}\n", entries[j].Fields_Values["title"]);
          repeated.push(j);
          }
        }
      }
    }  
  }
  println!("Removed {} entries that mirror reviewed ones", &repeated.len());
  repeated.sort_unstable_by(|a, b| b.cmp(a));
  repeated.dedup();

  for i in repeated{
    entries.remove(i);
  }

  // Remove identical entries
  let mut repeated: Vec<usize> = vec![];
  for i in 0..entries.len(){
    if entries[i].Files.len() > 0 && entries[i].Reviewed {
      for j in (i+1)..entries.len(){
        if entries[j].Files.len() > 0{
          if entries[i].Files == entries[j].Files{
            // println!("\n{}", entries[i].Fields_Values["title"]);
            // println!("{}\n", entries[j].Fields_Values["title"]);
            repeated.push(j);
          }
        }
      }
    }  
  }
  println!("Removed {} entries that point to the same file as reviewed ones", &repeated.len());
  repeated.sort_unstable_by(|a, b| b.cmp(a));
  repeated.dedup();

  for i in repeated{
    entries.remove(i);
  }

  // Remove identical entries
  let mut repeated: Vec<usize> = vec![];
  for i in 0..entries.len(){
    if entries[i].Fields_Values.contains_key("title") && entries[i].Files.len() > 0 {
      for j in (i+1)..entries.len(){
        if entries[j].Fields_Values.contains_key("title") && entries[j].Files.len() > 0 && !entries[j].Reviewed{
          let val_i: String = entries[i].Fields_Values["title"].trim().to_lowercase().chars().filter(|x| x.is_alphanumeric()).collect();
          let val_j: String = entries[j].Fields_Values["title"].trim().to_lowercase().chars().filter(|x| x.is_alphanumeric()).collect();
          if val_i == val_j && entries[i].Files == entries[j].Files {
            // println!("\n{}", entries[i].Fields_Values["title"]);
            // println!("{}\n", entries[j].Fields_Values["title"]);
            repeated.push(j);
          }
        }
      }
    }  
  }
  println!("Removed {} redundand entries", &repeated.len());
  repeated.sort_unstable_by(|a, b| b.cmp(a));
  repeated.dedup();

  for i in repeated{
    entries.remove(i);
  }

  // remove_by_field(entries, "title");
  // remove_by_field(entries, "file");// Check entries with same abstract
  // remove_by_field(entries, "abstract");
  remove_by_field(entries, "doi");
  // remove_by_field(entries, "isbn");// Check entries with same isbn
  // remove_by_field(entries, "url");// Check entries with same url
  // remove_by_field(entries, "booktitle");// Check entries with same booktitle
  // remove_by_field(entries, "eprint");// Check entries with same eprint
  // remove_by_field(entries, "arxivid");// Check entries with same arxivid  
  // remove_by_field(entries, "shorttitle");// Check entries with same shorttitle
  // remove_by_field(entries, "pmid");// Check entries with same pmid
}

fn remove_by_field(mut entries: & mut Vec<Entry>, field:&str){
  // Check entries with same doi
  let mut repeated: Vec<usize> = vec![];
  for i in 0..entries.len(){
    for j in (i+1)..entries.len(){
      if entries[i].Fields_Values.contains_key(field) && entries[j].Fields_Values.contains_key(field){
        let val_i: String = entries[i].Fields_Values[field].trim().to_lowercase().chars().filter(|x| x.is_alphanumeric()).collect();
        let val_j: String = entries[j].Fields_Values[field].trim().to_lowercase().chars().filter(|x| x.is_alphanumeric()).collect();
        if val_i == val_j{
          // println!("\n{}", entries[i].Fields_Values[field]);
          // println!("{}\n", entries[j].Fields_Values[field]);
          let merged = merge(&mut entries, i, j);
          if merged{
            repeated.push(j);
          }
        }
      }
    }
  }

  println!("Same {}, compatible {}", field, &repeated.len());
  repeated.sort_unstable_by(|a, b| b.cmp(a));
  repeated.dedup();

  for i in repeated{
    entries.remove(i);
  }
}

fn merge(entries: &mut Vec<Entry>, i: usize, j: usize) -> bool{
  // checking creators field
  let mut creators_i:HashSet<String> = HashSet::new();
  for (_, names) in &entries[i].Creators{
    for name in names{
      if !name.last_name.trim().is_empty() {
        creators_i.insert(
          name.last_name.chars().filter(|x| x.is_alphabetic()).collect()
        );
      }
      if !name.first_name.trim().is_empty() {
        creators_i.insert(
          name.last_name.chars().filter(|x| x.is_alphabetic()).collect()
        );
      }
    }
  }
  let mut creators_j:HashSet<String> = HashSet::new();
  for (_, names) in &entries[j].Creators{
    for name in names{
      if !name.last_name.trim().is_empty() {
        creators_j.insert(
          name.last_name.chars().filter(|x| x.is_alphabetic()).collect()
        );
      }
      if !name.first_name.trim().is_empty() {
        creators_j.insert(
          name.last_name.chars().filter(|x| x.is_alphabetic()).collect()
        );
      }
    }
  }

  if creators_i != creators_j {
    return false
  }

  let f1:HashSet<String> = entries[i].Fields_Values.iter().filter(|x| !x.1.trim().is_empty()).map(|x| x.0.to_owned()).collect();
  let f2:HashSet<String> = entries[j].Fields_Values.iter().filter(|x| !x.1.trim().is_empty()).map(|x| x.0.to_owned()).collect();
  let intersection = f1.intersection(&f2).to_owned();
  let common_fields:Vec<&String> = intersection.collect();
  for field in common_fields{
    let val_i: String = entries[i].Fields_Values[field].trim().to_lowercase().chars().filter(|x| x.is_alphanumeric()).collect();
    let val_j: String = entries[j].Fields_Values[field].trim().to_lowercase().chars().filter(|x| x.is_alphanumeric()).collect();
    if val_i != val_j {
      return false
    }
  }

  // entries are equivalent, merging
  if entries[j].Reviewed{
    entries[i].Reviewed = true;
    entries[i].Type = entries[j].Type.clone();
  }
  entries[i].Files = entries[i].Files.union(&entries[j].Files).map(|x| x.to_owned()).collect();
  entries[i].Tags = entries[i].Tags.union(&entries[j].Tags).map(|x| x.to_owned()).collect();
  entries[i].Tags.insert(MERGED.to_string());

  for field in f2.difference(&f1){
    let value = entries[j].Fields_Values.get_mut(field).unwrap().to_string();
    if let Some(x) = entries[i].Fields_Values.get_mut(field) {
      *x = value;
    }
  }
  true
}


#[test]
fn test_parse_creators_field(){
let input = "{Antonio Leiva},";
let output = "Antonio Leiva";
let vec = parse_creators_field(input);
let result = names_to_string(&vec);
for n in vec{
  println!("f:{}  l:{}", n.first_name, n.last_name);
}
assert_eq!(output, result);
}




fn read_and_parse_json(path: &Path) -> Vec<Entry>{
  let file = File::open(path).expect("File not found");
  serde_json::from_reader(file).expect("Error while reading")
}
#[test]
fn test_json(){

  let path = Path::new("tests/res.json");
  let entries = read_and_parse_json(path);
  let path2 = Path::new("tests/res2.json").to_owned();
  write_json(&path2, &entries);
  let entries2 = read_and_parse_json(&path2);
  assert_eq!(entries, entries2);
}

#[test]
fn test_bib(){
  let path = Path::new("tests/res.json");
  let baseentries = read_and_parse_json(path);

  let path2 = Path::new("tests/res1.bib").to_owned();
  write_bib(&path2, &baseentries);

  let mut lines: Vec<String> =vec![];
  read_bib(path2, &mut lines);
  let entries1 = parse_bib(&lines);

  let path3 = Path::new("tests/res2.bib").to_owned();
  write_bib(&path3, &entries1);

  let mut lines: Vec<String> =vec![];
  read_bib(path3, &mut lines);
  let entries2 = parse_bib(&lines);  

  // let path4 = Path::new("tests/res_from_bib.json").to_owned();
  // write_json(&path4, &entries2);
  assert_eq!(entries2, entries1, "Problem!");
}

#[test]
fn test_csv(){
  let path = Path::new("tests/res.json");
  let mut baseentries = read_and_parse_json(path);
  let stats =get_statistics_and_clean(&mut baseentries, false);

  let path2 = Path::new("tests/res1.csv").to_owned();
  write_csv(&path2, &baseentries, &stats.ordered_fields);


  let mut entries1 = read_and_parse_csv(path2);
  let stats =get_statistics_and_clean(&mut entries1, false);
  let path3 = Path::new("tests/res2.csv").to_owned();
  write_csv(&path3, &entries1, &stats.ordered_fields);

  let entries2 = read_and_parse_csv(path3);

  assert_eq!(entries2, entries1, "Problem!");
}

#[test]
fn test_bib_csv(){
  let path1 = Path::new("tests/res1.bib").to_owned();
  let mut lines: Vec<String> =vec![];
  read_bib(path1, &mut lines);
  let entries1 = parse_bib(&lines);

  let path2 = Path::new("tests/res1.csv").to_owned();
  let entries2 = read_and_parse_csv(path2);

  assert_eq!(entries2, entries1, "Problem!");
}

#[test]
fn remove_redundant(){
  let path = Path::new("tests/tomerge.csv").to_owned();
  let mut entries = read_and_parse_csv(path);

  let path = Path::new("tests/tomerge.bib").to_owned();
  write_bib(&path, &entries);

  remove_redundant_Entries(&mut entries);

  let path = Path::new("tests/tomerge_out.bib").to_owned();
  write_bib(&path, &entries);

  // assert_eq!(entries2, entries1, "Problem!");
}







// fn get_files_from_paths(entries: &mut Vec<Entry>, doc_paths: &Vec<PathBuf>){
//   println!("Fetching files in disc:");
//   let mut filename_path: HashMap<String, PathBuf> = HashMap::new();
//   for p in doc_paths{
//     filename_path.insert(
//       p.file_name().unwrap().to_str().unwrap().to_string().replace(":", ""),
//       p.to_path_buf()
//     );
//   }

//   for e in entries{
//     if !e.BrokenFiles.is_empty() {
//       for p in &e.BrokenFiles{
//         let filename = p.split("/").last().unwrap();
//         if filename_path.contains_key(filename){
//           // println!("{}", filename);
//           // e.has_file = true;
//           e.Files.insert(filename_path[filename].as_path().to_str().unwrap().to_string());
//         }
//       }
//     }
//   }
// }




// fn write_raw_bib(path: &str, bib_vec : &Vec<String>){
//   let path = Path::new(path);
//   let display = path.display();

//   // Open a file in write-only mode, returns `io::Result<File>`
//   let mut f = match File::create(&path) {
//       Err(why) => panic!("couldn't create {}: {}", display, why),
//       Ok(file) => file,
//   };

//   for l in bib_vec{
//     writeln!(f, "{}",l).unwrap();
//   }
// }
