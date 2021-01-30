#![feature(str_split_once)]
#![allow(non_snake_case)]

use std::{cmp::Ordering, vec};
use std::io::BufReader;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::collections::HashSet;
use std::str;
use std::env;
use std::ffi::OsStr;
extern crate csv;

extern crate clap;
use clap::{App, Arg, ArgMatches};

extern crate walkdir;
use walkdir::WalkDir;


static INTERNAL_TAG_MARKER: char ='#';
static REVIEWED: &str = "#reviewed";
static MERGED: &str = "#merged";

static SEPARATOR: &str = ",";
static INTERNAL_SEPARATOR: &str = ",";
static NAMES_SEPARATOR: &str = " and ";

fn get_statistics(Entries:&Vec<Entry>) -> (Vec<String>,Vec<String>){
  let mut types: HashMap<String, u32> = HashMap::new();
  let mut fields: HashMap<String, u32> = HashMap::new();
  let mut has_doi:usize = 0;
  let mut has_file:usize = 0;
  let mut has_url:usize = 0;
  let mut has_author:usize = 0;
  let mut reviewed:usize = 0;

  for entry in Entries{
    if entry.Tags.contains(REVIEWED){
      reviewed += 1;
    }
    if types.contains_key(&entry.Type){
      *types.get_mut(&entry.Type).unwrap() += 1;
    }else{
      types.insert(entry.Type.to_string(), 1);
    }

    if entry.Files.len() > 0{
      has_file += 1;
    }

    if entry.Creators.len() > 0{
      has_author += 1;
    }
  
    for (field, _) in &entry.Fields_Values{
      if fields.contains_key(field){
        *fields.get_mut(field).unwrap() += 1;
        match field.as_ref() {
          "doi" => has_doi += 1,
          "url" => has_url += 1,
          _ => continue ,

        }
      }else{
        fields.insert(field.to_string(), 1);
      }
    }
  }
  
  // Sorting
  let mut types_vec: Vec<(String, u32)> = vec![];

  for (t,c) in types{
    types_vec.push((t.to_string(), c));
  }  

  println!(
    "\nFound a total of {} entries\n({} reviewed, {} with author, {} with doi, {} with files, {} whith url):", 
                        Entries.len(), reviewed, has_author, has_doi, has_file, has_url);
    
  let mut ordered_types:Vec<String> = Vec::new();
  types_vec.sort_by(|a, b| a.1.cmp(&b.1).reverse());
  for (key, value) in types_vec {
    println!("{} {}", key, value);
    ordered_types.push(key);
  }

  println!("\nFields:");
  let mut fields_vec: Vec<(String, u32)> = vec![];

  for (t,c) in fields{
    fields_vec.push((t.to_string(), c));
  }  

  let mut ordered_fields:Vec<String> = Vec::new();

  fields_vec.sort_by(|a, b| a.1.cmp(&b.1).reverse());
  for (key, value) in fields_vec {
    println!("{} {}", key, value);
    ordered_fields.push(key);
  }
  println!("");
  (ordered_types, ordered_fields)  
}

#[derive(PartialEq, Clone)]
struct Name{
  first_name:String,
  last_name:String
}

struct Entry {
  Type: String,
  Key: String,
  Creators: HashMap<String, Vec<Name>>,
  Tags:HashSet<String>,
  Files: HashSet<String>,

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
      let t = v.iter().map(|x| format!("{} {}", x.first_name, x.last_name).trim().to_string()).collect::<Vec<String>>().join(" and ");
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
    let t = e.Files.iter().map(|x| x.replace(":", "\\:")).collect::<Vec<String>>().join(";");
    s.push_str(&format!("file = {{{}}},\n", t));
  }

  //Tags
  if !e.Tags.is_empty() {
    let t = e.Tags.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(",");
    s.push_str(&format!("mendeley-tags = {{{}}},\n", t));
  }
  s.push_str("}\n");
  s
}

fn create_Entry(Type:String, Key:String) -> Entry{
  Entry{
    Type:Type, 
    Key: Key,
    Creators: HashMap::new(),
    Fields_Values: HashMap::new(),
    Files: HashSet::new(),
    Tags: HashSet::new(),
  }
}

fn read_bib(path:PathBuf, bib_lines:&mut Vec<String>){
  let file = File::open(path).unwrap();
  let file_buffer = BufReader::new(file);

  let mut inside_comment=false;
  for line in file_buffer.lines(){
    let l = line.unwrap();

    if std::str::from_utf8(l.as_bytes()).is_err(){
      println!("utf8 error in {}", l);
    }

    let l = l.trim();
    if l.to_lowercase().starts_with("@comment"){
      inside_comment = true;
    }
    if !l.is_empty() && !inside_comment && !l.starts_with('%'){
      bib_lines.push(l.to_string().replace("“", "\"").replace("”", "\""));
    }
    if inside_comment && l.ends_with('}'){
      inside_comment = false;
    }
  }
}

fn parse_creators_field(original_value:&str) -> Vec<Name>{
  let mut authors: Vec<Name> = vec![];
  // let patterns : &[_] = &['{', '}','\t',',',' '];
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
  for fl in original_value.split("and").map(|x| x.trim()){
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

fn parse_tags_field(original_value:&str) -> HashSet<String>{
  // let patterns : &[_] = &['{', '}','\t',',',' ',','];
  let tags: HashSet<String> = 
    original_value
      .replace(";", ",")
      .split(",").map(|x| x.to_lowercase().trim_matches(|c:char| c != INTERNAL_TAG_MARKER && !c.is_alphabetic()).to_owned())
      .filter(|s| !s.is_empty())
      .collect();
  tags
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
          // println!("{}", p);
          if e.Fields_Values.contains_key("broken-files"){
            e.Fields_Values.get_mut("broken-files").unwrap().push_str(&format!(",{}", p).to_string());
          }
          else{
            e.Fields_Values.insert("broken-files".to_string(), p);
          }
        }
      }
    }
  checked
}

fn parse_generic_field(original_value:&str) -> String{
  let patterns : &[_] = &['\t',',',' '];
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
    if lines[counter].starts_with("@"){ // found entry
      let vec: Vec<&str> = lines[counter].splitn(2,"{").collect();
      if vec.len() == 2 {
        let Type =vec[0].trim().trim_matches('@').to_lowercase();
        let Key =vec[1].trim().trim_matches(',');
        Entries.push(create_Entry(Type, Key.to_string())) ;
      }
      else{
        println!("Problem: {}\n",lines[counter]);
      }
      counter +=1;
      while counter < lines.len() && lines[counter].trim() != "}"{ // while inside entry
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
            "file" => last_entry.Files = parse_file_field(&mut last_entry, &value.to_string()), //parse_file_field(value),
            "author" | "editor" | "translator" => {
              let _ = last_entry.Creators.insert(field.to_string(), parse_creators_field(&value));
            },
            "mendeley-tags"|"groups"|"tags" => last_entry.Tags = parse_tags_field( &value),
            _ => {

              if field == "isbn" {
                value = value.chars().filter(|x| x.is_numeric()).collect()
              };

              if field == "arxivid" || field == "eprint" {
                value = value.split(":").map(|x| x.to_string()).collect::<Vec<String>>().last().unwrap().to_string()
              };

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

fn hashset_to_string(files: &HashSet<String>) -> String{
  files.to_owned().into_iter().collect::<Vec<String>>().join(INTERNAL_SEPARATOR)
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
    let c0:String = match e.Tags.contains(REVIEWED){
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
    writeln!(f, "{}",row.join(",")).unwrap();
  }
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

fn find_paths_to_files_with_ext(root_path:&str, exts:& Vec<String> ) -> Vec<PathBuf>{
  let mut paths: Vec<PathBuf> = vec![];
  for direntry in WalkDir::new(root_path).into_iter().filter_map(|e| e.ok()){
    let ext = direntry.path().extension();
    if ext.is_some() && exts.contains(&ext.unwrap().to_str().unwrap().to_lowercase()){
      paths.push(direntry.path().to_owned());
    }
  }
  paths
}

fn get_entries_from_root_path(root_path:String) -> Vec<Entry>{
  let exts=vec!["bib".to_string()];
  let mut bib_paths = find_paths_to_files_with_ext(&root_path, &exts);

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
    let mut e = create_Entry(v[1].to_owned(), v[2].to_owned());
    // println!("{:?}\n", v);
    // creators
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
      e.Tags = parse_tags_field(&v[n-2]);
      if !v[0].is_empty(){
        e.Tags.insert(REVIEWED.to_string());
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
    .get_matches()
}

fn from_path_to_entries(input_path: String) -> Option<Vec<Entry>>{
  let mut main_entries = vec![];
  let p = PathBuf::from(input_path);
  if p.is_dir(){
    println!("Searching for .bib files in {:#?}:", p);
    main_entries = get_entries_from_root_path("bibs/main".to_string());
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

fn main() {
  let args = parse_cl_args();

  // read input path from args
  let mut input_path = String::new();
  if args.value_of("input").is_some(){
    input_path = args.value_of("input").unwrap().to_string();
  }
  else if args.value_of("input_pos").is_some() {
    input_path = args.value_of("input_pos").unwrap().to_string();
  }
  else{
    input_path = env::current_dir().unwrap().to_str().unwrap().to_string();
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
    let files_root_path = args.value_of("files").unwrap().to_string();
    let exts = vec!["pdf".to_string(),"epub".to_string(),"djvu".to_string()];
    let filepaths = find_paths_to_files_with_ext(&files_root_path, &exts);
    if !filepaths.is_empty() {
      get_files_from_paths(&mut main_entries, &filepaths);
    }
  }

  // output
  let (_, ordered_fields) = get_statistics(&main_entries);

  let mut output_path = String::new();
  if args.value_of("output").is_some(){
    output_path = args.value_of("output").unwrap().to_string();
  }
  else if args.value_of("output_pos").is_some() {
    output_path = args.value_of("output_pos").unwrap().to_string();
  }
  else{
    output_path = env::current_dir().unwrap().to_str().unwrap().to_string();
  }

  let mut p = PathBuf::from(output_path);
  if p.is_dir(){
    p.push("Result.csv");
    println!("Saving results at {:#?}:", p);
    write_csv(&p, &main_entries, &ordered_fields)
  }
  else if p.extension().is_some(){
    match p.extension().unwrap().to_str() {
      Some("csv")    => write_csv(&p, &main_entries, &ordered_fields),
      Some("bib")    => write_bib(&p, &main_entries),
      Some(ext) => {
        p.set_extension("csv");
        write_csv(&p, &main_entries, &ordered_fields)
      },
      None           => println!("No extension detected in {:?}", p.as_os_str()),
    }
  }
  else{
    p = env::current_dir().unwrap();
    p.push("Result.csv");
    write_csv(&p, &main_entries, &ordered_fields);
  }

  


  // let mut main_entries = get_entries_from_root_path("bibs/main".to_string());
  // remove_redundant_Entries(& mut main_entries);

  // let mut doc_paths: Vec<PathBuf> = vec![];
  // find_paths_to_files_with_ext(
  //   "C:/Users/tesse/Desktop/Files/Dropbox/BIBrep",
  //   &mut doc_paths,
  //   &vec!["pdf".to_string(),"epub".to_string(),"djvu".to_string()]
  // );

  // get_files_from_paths(&mut main_entries, &doc_paths);

  // let mut other_entries = get_entries_from_root_path("bibs/other".to_string());
  // remove_redundant_Entries(&mut other_entries);

  // get_files_from_entries(&mut main_entries, &other_entries);

  // let (_, ordered_fields) = get_statistics(&main_entries);



  // main_entries.sort();
  // write_bib("Complete.bib", &main_entries);
  // write_csv("Complete.csv", &main_entries, &ordered_fields);

  // let mut e1 = read_and_parse_csv("Complete.csv".to_string());


  // e1.sort();
  // write_bib("Complete_from_csv.bib", &e1);

  // let mut all: Vec<Entry> = vec![];
  // all.append(&mut main_entries);
  // all.append(&mut e1);

  // println!("before: {}", all.len());
  // remove_redundant_Entries(&mut all);
  // println!("after: {}", all.len());
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
  let mut filename_path: HashMap<String, PathBuf> = HashMap::new();
  for p in doc_paths{
    filename_path.insert(
      p.file_name().unwrap().to_str().unwrap().to_string().replace(":", ""),
      p.to_path_buf()
    );
  }

  for e in entries{
    if e.Fields_Values.contains_key("broken-files") {
      for p in e.Fields_Values["broken-files"].split(";"){
        let filename = p.split("/").last().unwrap();
        if filename_path.contains_key(filename){
          println!("{}", filename);
          // e.has_file = true;
          e.Files.insert(filename_path[filename].as_path().to_str().unwrap().to_string());
        }
      }
      e.Fields_Values.remove("broken-files");
    }
  }



  // for e in entries{
  //   let mut checked: Vec<String> = vec![];
  //   if e.Files.len() > 0 {
  //     for raw_f in e.Files.to_owned(){
  //       let paths = raw_f
  //         .split(":")
  //         .filter(|x| !x.is_empty() && x.contains("."))
  //         .map(|x| format!("C:{}", x.replace("\\\\", "/").replace("\\", "/")))
  //         .collect::<Vec<String>>();
  //         for p in paths{
  //           if Path::new(&p).exists(){
  //             // println!("{}", p);
  //             e.has_file = true;
  //             checked.push(Path::new(&p).as_os_str().to_str().unwrap().to_string())
  //           }
  //           else{
  //             let filename = p.split("/").last().unwrap();
  //             if filename_path.contains_key(filename){
  //               // println!("{}", filename);
  //               e.has_file = true;
  //               checked.push(filename_path[filename].as_path().to_str().unwrap().to_string())
  
  //             }
  //           }
  //         }
  //     }

  //     // else if e.Authors.len()>0 && e.Fields_Values.contains_key("year") && e.Fields_Values.contains_key("title"){
  //     //   let t = format!(
  //     //     "{} {} - {}{}.pdf",
  //     //     e.Fields_Values["year"],
  //     //     e.Fields_Values["title"],
  //     //     &e.Authors[0].last_name,
  //     //     if e.Authors.len() > 1 { " et al" } else { "" }
  //     //   );
  //     //   // println!("{}", t);
  //     //     if doc_paths.contains(&t){
  //     //       e.has_file = true;
  //     //       println!("{}", t);
  //     //     }
  //     // }
  //   }
  //   e.Files = checked;
  // }
}

fn remove_redundant_Entries(entries: & mut Vec<Entry>){
  // Remove identical entries
  let mut repeated: Vec<usize> = vec![];
  for i in 0..entries.len(){
    for j in i+1..entries.len(){
      if entries[i] == entries[j] {
        repeated.push(j);
      }
    }
  }  
  println!("Removed {} Identical entries\n", &repeated.len());
  repeated.sort();
  repeated.reverse();
  for i in repeated{
    entries.remove(i);
  }
  
  remove_by_field( entries, "doi");// Check entries with same doi
  remove_by_field( entries, "isbn");// Check entries with same isbn
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
    for j in i+1..entries.len(){
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
  repeated.sort();
  repeated.reverse();
  for i in repeated{
    entries.remove(i);
  }
}

fn merge(entries: &mut Vec<Entry>, i: usize, j: usize) -> bool{
  if entries[i].Creators != entries[j].Creators {
    return false
  }
  let f1:HashSet<String> = entries[i].Fields_Values.iter().map(|x| x.0.to_owned()).collect();
  let f2:HashSet<String> = entries[j].Fields_Values.iter().map(|x| x.0.to_owned()).collect();
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
    entries[i].Files = entries[i].Files.union(&entries[j].Files).map(|x| x.to_owned()).collect();
    entries[i].Tags = entries[i].Tags.union(&entries[j].Files).map(|x| x.to_owned()).collect();
    entries[i].Tags.insert(MERGED.to_string());



    for field in f2.difference(&f1){
      let value =entries[j].Fields_Values.get_mut(field).unwrap().to_string();
      if let Some(x) = entries[i].Fields_Values.get_mut(field) {
        *x = value;
      }
    }
  }
  eq
}

// fn write_sqlite(path: &str, entries: &Vec<Entry>, ordered_fields: &Vec<String>){
//   let path = Path::new(path);
//   // let display = path.display();

//   // Open a file in write-only mode, returns `io::Result<File>`
//   let conn = Connection::open(&path).unwrap();

//   let res = conn.execute(
//     "CREATE TABLE IF NOT EXISTS Entries (
//       type           TEXT,
//       [key]          TEXT PRIMARY KEY
//                      NOT NULL
//                      UNIQUE,
//       author         TEXT,
//       title          TEXT UNIQUE
//                           NOT NULL,
//       year           INT  CHECK (length(year) == 2 OR 
//                                  length(year) == 4),
//       pages          TEXT,
//       abstract       TEXT,
//       doi            TEXT UNIQUE,
//       volume         TEXT,
//       issn           TEXT,
//       journal        TEXT,
//       number         TEXT,
//       keywords       TEXT,
//       publisher      TEXT,
//       isbn           TEXT UNIQUE,
//       url            TEXT,
//       booktitle      TEXT,
//       shorttitle     TEXT,
//       eprint         TEXT UNIQUE,
//       archiveprefix  TEXT,
//       arxivid        TEXT UNIQUE,
//       pmid           TEXT,
//       annote         TEXT,
//       edition        TEXT,
//       address        TEXT,
//       typefield      TEXT,
//       month          INT,
//       series         TEXT,
//       editor         TEXT,
//       institution    TEXT,
//       howpublished   TEXT,
//       organization   TEXT,
//       school         TEXT,
//       translator     TEXT,
//       [broken-files] TEXT,
//       qualityassured TEXT,
//       tags           JSON,
//       file           JSON
//   )
//   WITHOUT ROWID;"
//      ,NO_PARAMS,
//     );

//     match res {
//       Ok(v) => println!("table created: {:?}", v),
//       Err(e) => println!("sqlite error: {:?}", e),
//   }

//   for e in entries{
//     let c = e.Fields_Values.to_owned().into_iter().map(|x| x.0).collect::<Vec<String>>().join(",");
//     let columns = format!("type,key,author,file,tags,{}", c);

//     let a = e.Authors.iter().map(|x| format!("{} {}", x.first_name, x.last_name).trim().to_string()).collect::<Vec<String>>().join(" and ");
//     let f = files_to_string(&e.Files);
//     let t = e.Tags.to_owned().into_iter().collect::<Vec<String>>().join(";");
//     let o = e.Fields_Values.iter().map(|x| format!("\"{}\"", x.1)).collect::<Vec<String>>().join(",");
//     let values = format!("\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",{}",e.Type, e.Key, a, f, t, o);

//     let instruction = format!("INSERT INTO Entries ({})\nVALUES({});", columns, values);
//     let res = conn.execute(&instruction, NO_PARAMS);
//     // let res = conn.execute(&format!("json_array({})", f).to_string(), NO_PARAMS);
//     // println!("{}\n", instruction);
//     match res {
//       Ok(v) => continue, //println!("row inserted: {:?}", v),
//       // Err(e) => println!("{}\n{:?}\n", instruction, e),
//       Err(e) => println!("{:?}", e),
//     }
//   }
// }

// fn inspect_entries(entries: &mut Vec<Entry>){
//   for e in entries{
//     // check author field
//     if e.Creators.is_empty() {
//       e.Tags.insert("#no author".to_string());
//     }
//     else if !e.Creators
//               .iter()
//               .map(|x| format!("{} {}", x.first_name, x.last_name))
//               .collect::<Vec<String>>()
//               .join("")
//               .replace(".", "")
//               .replace(" ", "")
//               .chars().all(char::is_alphanumeric)
//     {
//       e.Tags.insert("#corrupted author".to_string());
//     }
//   }
// }

// fn write_to_ads(){
//   let path = Path::new("ads_test.txt:ads.bib");
//   let display = path.display();

//   // Open a file in write-only mode, returns `io::Result<File>`
//   let mut f = match File::create(&path) {
//       Err(why) => panic!("couldn't create {}: {}", display, why),
//       Ok(file) => file,
//   };


//   writeln!(f, "test 01\ntest 02");
// }