#![feature(str_split_once)]
#![allow(non_snake_case)]

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
extern crate csv;

extern crate clap;
use clap::{App, Arg, ArgMatches, Result};

extern crate walkdir;
use walkdir::WalkDir;

extern crate url;
use url::Url;

extern crate nom_bibtex;
use nom_bibtex::*;

extern crate serde;
use serde::{Deserialize, Serialize};

extern crate html_minifier;
use html_minifier::HTMLMinifier;

// extern crate minifier;
// use minifier::js::minify;

static INTERNAL_TAG_MARKER: char ='#';
static REVIEWED: &str = "#reviewed";
static MERGED: &str = "#merged";

static SEPARATOR: &str = ",";
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
    for (c, v) in &e.Creators{
      let t = names_to_string(v);
      s.push_str(&format!("{} = {{{}}},\n", c, t));
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
      println!("{}", l);

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
  for fl in original_value.trim_matches(pattern).split("and").map(|x| x.trim()){
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

fn parse_generic_field(original_value:&str) -> String{
  let patterns : &[_] = &['\t',',',' ','"','\''];
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
          match field {
            "file" => last_entry.Files = parse_file_field(&mut last_entry, &value.to_string()),
            "author" | "editor" | "translator" => {
              let _ = last_entry.Creators.insert(field.to_string(), parse_creators_field(&value));
            },
            "mendeley-groups"|"mendeley-tags"|"groups"|"tags" => parse_tags_field( &mut last_entry,&value),
            _ => {

              if field == "isbn" {
                value = value.chars().filter(|x| x.is_numeric()).collect()
              }
              else if field == "arxivid" || field == "eprint" {
                value = value.split(":").map(|x| x.to_string()).collect::<Vec<String>>().last().unwrap().to_string()
              }
              else if field == "url"{
                value = parse_url_field(&value);
              }

              if Entries.last().unwrap().Fields_Values.contains_key(field){
                println!("Repeated entry at {}\n", field_value);
              }
              else{
                Entries.last_mut().unwrap().Fields_Values.insert(field.to_string(), parse_generic_field(&value));
              }
            }
          };
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

  write!(f,"\u{feff}"); // BOM, indicating uft8 for excel

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
  let mut html = String::new();
  let html_template = std::fs::read_to_string("deps/table.html").expect("Something went wrong reading table.html");
  let css = std::fs::read_to_string("deps/01table.css").expect("Something went wrong reading 01table.css");
  let mut js = std::fs::read_to_string("deps/02tabulator.js").expect("Something went wrong reading 02tabulator.js");
  let table = std::fs::read_to_string("deps/04table.js").expect("Something went wrong reading 04table.js");

  // let mut js = tabulator;
  js.push_str(&entries_to_js_obj(entries));
  js.push_str((&table));
  // js = minify(&js);

  html = html_template
    .replace(
    "  <link href=\"01table.css\" rel=\"stylesheet\">",
     &format!("<style>\n{}\n</style>", css)
    )
    .replace(
      "  <script type=\"text/javascript\" src=\"02tabulator.js\"></script>",
      "")
    .replace(
      "  <script type=\"text/javascript\" src=\"03result.js\"></script>",
      ""
    )
    .replace(
      "  <script src=\"04table.js\"></script>",
      &format!("<script>\n{}\n</script>", js)
    );
    

  let display = path.display();

  let mut f = match File::create(&path) {
      Err(why) => panic!("couldn't create {}: {}", display, why),
      Ok(file) => file,
  };

  let mut html_minifier = HTMLMinifier::new();
  match html_minifier.digest(html) {
    Ok(m)  => (),//println!("{:?}", m),
    Err(e) => println!("{:?}", e),
  }

  f.write_all(html_minifier.get_html()).expect("unable to write html to disk");
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

fn write_raw_bib(path: &str, bib_vec : &Vec<String>){
  let path = Path::new(path);
  let display = path.display();

  // Open a file in write-only mode, returns `io::Result<File>`
  let mut f = match File::create(&path) {
      Err(why) => panic!("couldn't create {}: {}", display, why),
      Ok(file) => file,
  };

  for l in bib_vec{
    writeln!(f, "{}",l).unwrap();
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
  let n = keys.len();
  println!("{:?}", keys);
  let mut Entries : Vec<Entry> = vec![];

  while let Some(result) = rdr.records().next() {
    match result {
      Ok(_) => (),
      Err(e) =>{
        println!("{:?}\n", e);
        continue
      },
    }
    let v:Vec<String> = result.unwrap().into_iter().map(|x| x.to_string()).collect();
    let mut e = Entry{Type: v[1].to_owned(), Key: v[2].to_owned(),..Default::default()};
    if !v[3].trim().is_empty(){
      e.Creators.insert("author".to_string(), parse_creators_field(&v[3]));
    }
    if !v[4].trim().is_empty(){
      e.Creators.insert("editor".to_string(), parse_creators_field(&v[4]));
    }
    if !v[5].trim().is_empty(){
      e.Creators.insert("translator".to_string(), parse_creators_field(&v[5]));
    }

    if !v[0].trim().is_empty() || !v[n-2].is_empty(){
      parse_tags_field(&mut e,&v[n-2]);
      if !v[0].is_empty(){
        e.Reviewed=true;
      }
    }

    for j in 6..n-2{
      if !v[j].trim().is_empty(){
        e.Fields_Values.insert(keys[j].to_owned(), v[j].to_owned());
      }
    }

    if !v[n-1].trim().is_empty(){
      e.Files = parse_file_field(&mut e,&v[n-1]);
    }
    Entries.push(e);
  }
  Entries
}

fn read_and_parse_json(path: &Path) -> Vec<Entry>{
  let file = File::open(path).expect("file not found");
  serde_json::from_reader(file).expect("error while reading")
}

fn parse_cl_args() -> ArgMatches<'static> {
  App::new("BibRust")
    .version("1.0")
    .about("Tame your (bib)liographic files!")
    .author("https://carlos-tarjano.web.app/")
    .arg(Arg::with_name("input_pos")
      .short("OP")
      .long("INPUT")
      .value_name("INPUT")
      .help("positional: The input file to convert from (.bib or .csv), or the root folder in which to search for .bib files")
      .takes_value(true)
      .required(false)
      .index(1)
      .conflicts_with("input")
    )
    .arg(Arg::with_name("output_pos")
      .short("OP")
      .long("OUTPUT")
      .value_name("OUTPUT")
      .help("positional: Path[s] to output file[s] (.bib and / or .csv)")
      .takes_value(true)
      .required(false)
      .index(2)
      .conflicts_with("output")
    )
    .arg(Arg::with_name("input")
      .short("i")
      .long("input")
      .value_name("path to file or folder")
      .help("The input file to convert from (.bib or .csv), or the root folder in which to search for .bib files")
      .takes_value(true)
      .required(false)
    )
    .arg(Arg::with_name("output")
      .short("o")
      .long("output")
      .value_name("path to file")
      .help("Path[s] to output file[s] (.bib and / or .csv)")
      .takes_value(true)
      .required(false)
    )
    .arg(Arg::with_name("auxiliary")
      .short("a")
      .long("auxiliary")
      .value_name("path to file or folder")
      .help("The file (.bib or .csv), or the root folder in which to search for .bib files to be used to complement the info")
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
    .arg(Arg::with_name("keys")
      .short("k")
      .long("keys")
      .value_name("update keys?")
      .help("if set, new unique keys will be created as [year]_[first author last name]_[number]")
      .takes_value(false)
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
  if entry.Fields_Values.contains_key("year"){
    year = entry.Fields_Values["year"].to_owned();
  }
  else if entry.Fields_Values.contains_key("date"){
    let optyear = entry.Fields_Values["date"].split( '-').next();
    year = match optyear {
      Some(y) => y.to_string(),
      None => "year".to_string()
    }
  }

  let mut roles: Vec<String> = vec![];
  for (c, names) in &entry.Creators{
    if !names.is_empty(){
      roles.push(c.to_owned());
    }
  }
  roles.sort();
  let mut etal = "";
  let creator = match roles.first() {
    Some(n) => {
      if entry.Creators[n].len() > 1 {
        etal = "_etal";
      }
      &entry.Creators[n][0].last_name
    }
    None => "creator",
  };

  format!("{}_{}{}", year, creator.chars().filter(|x| x.is_alphabetic()).collect::<String>().to_lowercase(), etal)
}

fn get_statistics(Entries:&mut Vec<Entry>, tidy:bool) -> Statistics{
  if tidy{
    println!("Generating keys:");
  }
  let mut stats = Statistics{..Default::default()};
  let n = Entries.len();
  for entry in Entries{
    if tidy{
      let base_key = generate_key(entry);
      let mut ctr = 0;
      let mut key = base_key.clone();
      while stats.keys.contains(&key) {
        ctr +=1;
        key = format!{"{}{}", base_key, ctr};
      }
      stats.keys.insert(key.clone());
      entry.Key = key;
    }

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
  
    for (field, _) in &entry.Fields_Values{
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

  fields_vec.sort_by(|a, b| a.1.cmp(&b.1).reverse());
  for (key, value) in fields_vec {
    println!("{} {}", key, value);
    stats.ordered_fields.push(key);
  }
  println!("");
  stats
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
      let key = generate_key(e);
      let typ = match e.Type.as_ref() {
        "article" | "inproceedings" | "incollection" => "a",
        "book" | "collection" | "thesis" | "mvbook" | "phdthesis" => "b",
        _ => "r"
      };
      let tit = match e.Fields_Values["title"].split(":").next(){
        Some(t) => t,
        None => "NO_TITLE"
      };
      println!("!{} {{{}}} {}", typ, key, tit);
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


  // file paths
  if args.value_of("files").is_some(){
    let files_root_path = PathBuf::from(args.value_of("files").unwrap().to_string());
    let exts = vec!["pdf".to_string(), "epub".to_string(), "djvu".to_string()];
    let filepaths = find_paths_to_files_with_ext(&files_root_path, &exts);
    if !filepaths.is_empty() {
      get_files_from_paths(&mut main_entries, &filepaths);
    }
  }


  // merge
  if args.is_present("merge"){
    remove_redundant_Entries(&mut main_entries);
  }


  // tidy
  let mut generate_keys = false;
  if args.is_present("keys"){
    generate_keys = true;
  }
  let stats = get_statistics(&mut main_entries, generate_keys);

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

fn get_files_from_paths(entries: &mut Vec<Entry>, doc_paths: &Vec<PathBuf>){
  println!("Fetching files in disc:");
  let mut filename_path: HashMap<String, PathBuf> = HashMap::new();
  for p in doc_paths{
    filename_path.insert(
      p.file_name().unwrap().to_str().unwrap().to_string().replace(":", ""),
      p.to_path_buf()
    );
  }

  for e in entries{
    if !e.BrokenFiles.is_empty() {
      for p in &e.BrokenFiles{
        let filename = p.split("/").last().unwrap();
        if filename_path.contains_key(filename){
          // println!("{}", filename);
          // e.has_file = true;
          e.Files.insert(filename_path[filename].as_path().to_str().unwrap().to_string());
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
      if entries[i] == entries[j] {
        repeated.push(j);
      }
    }
  }  
  println!("Removed {} Identical entries\n", &repeated.len());
  repeated.sort_unstable_by(|a, b| b.cmp(a));
  repeated.dedup();

  for i in repeated{
    entries.remove(i);
  }

  remove_by_field(entries, "doi");// Check entries with same doi
  remove_by_field(entries, "isbn");// Check entries with same isbn
  // remove_by_field( entries, "url");// Check entries with same url
  // remove_by_field( entries, "shorttitle");// Check entries with same shorttitle
  // remove_by_field( entries, "pmid");// Check entries with same pmid
  // remove_by_field( entries, "abstract");// Check entries with same abstract
  // remove_by_field( entries, "eprint");// Check entries with same eprint
  // remove_by_field( entries, "arxivid");// Check entries with same arxivid
  // println!("");
}

fn remove_by_field(mut entries: & mut Vec<Entry>, field:&str){
  // Check entries with same doi
  let mut repeated: Vec<usize> = vec![];
  for i in 0..entries.len(){
    for j in (i+1)..entries.len(){
      if 
      entries[i].Fields_Values.contains_key(field) &&
      entries[j].Fields_Values.contains_key(field) &&
      entries[i].Fields_Values[field].to_lowercase() == entries[j].Fields_Values[field].to_lowercase()
      {
        let merged = merge(&mut entries, i, j);
        if merged{
          repeated.push(j);
        }
      }
    }
  }

  println!("Same {}, compatible {}\n", field, &repeated.len());
  repeated.sort_unstable_by(|a, b| b.cmp(a));
  repeated.dedup();

  for i in repeated{
    entries.remove(i);
  }
}

fn merge(entries: &mut Vec<Entry>, i: usize, j: usize) -> bool{
  if entries[i].Creators != entries[j].Creators {
    return false
  }
  let f1:HashSet<String> = entries[i].Fields_Values.iter().filter(|x| !x.1.trim().is_empty()).map(|x| x.0.to_owned()).collect();
  let f2:HashSet<String> = entries[j].Fields_Values.iter().filter(|x| !x.1.trim().is_empty()).map(|x| x.0.to_owned()).collect();
  let intersection = f1.intersection(&f2).to_owned();
  let common_fields:Vec<&String> = intersection.collect();
  let mut eq = true;
  for field in common_fields{
    if entries[i].Fields_Values[field].to_lowercase() != entries[j].Fields_Values[field].to_lowercase(){
      eq = false;
      break;
    }
  }

  if eq{
    if entries[j].Reviewed{
      entries[j].Reviewed = true;
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
  }
  eq
}

fn parse_bib_nom_bibtex(lines:&Vec<String> ) -> Vec<Entry>{

  let bibtex = Bibtex::parse(&lines.join("\n\n")).unwrap();
  let mut Entries : Vec<Entry> = vec![];
  // let patterns : &[_] = &['{', '}','\t',','];

  for e in bibtex.bibliographies(){
    let mut entry = Entry{Type:e.entry_type().to_string(), Key:e.citation_key().to_string(),..Default::default()};
    for (field, mut value) in e.tags().to_owned(){
      match field.as_ref() {
        "file" => {parse_file_field(&mut entry, &value.to_string());},
        "author" | "editor" | "translator" => {
            entry.Creators.insert(field.to_string(), parse_creators_field(&value));
          },
        "mendeley-tags"|"groups"|"tags" => parse_tags_field( &mut entry,&value),
        _ => {
          match field.as_ref() {
            "isbn" => value = value.chars().filter(|x| x.is_numeric()).collect::<String>(),
            "arxivid" | "eprint" => value = value.split(":").map(|x| x.to_string()).collect::<Vec<String>>().last().unwrap().to_string(),
            "url" => value = parse_url_field(&value),
            _ => value = parse_generic_field(&value),
          }
          if entry.Fields_Values.contains_key(&field){
            println!("Repeated entry at {}\n", e.citation_key());
          }
          else{
            entry.Fields_Values.insert(field.to_string(), value.clone());
          }
        }
      }
    }
    Entries.push(entry);
  }
  Entries
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
  let mut entries = read_and_parse_json(path);
  let path2 = Path::new("tests/res.bib").to_owned();
  write_bib(&path2, &entries);
  let mut lines: Vec<String> =vec![];
  read_bib(path2, &mut lines);
  let mut entries2 = parse_bib(&lines);
  // entries.sort();
  // entries2.sort();
  let path3 = Path::new("tests/res_from_bib.json").to_owned();
  write_json(&path3, &entries2);
  assert_eq!(entries, entries2, "Problem!");
}