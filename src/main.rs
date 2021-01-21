// extern crate pdf_extract;
// extern crate lopdf;
// use pdf_extract::*;
// use lopdf::*;
// use std::path;
// use std::env;
// let path_to_pdf = path::Path::new("biblatex.pdf");
// let result = extract_text(path_to_pdf);

use std::{char::ToLowercase, hint::unreachable_unchecked, io, iter::{Map, Rev}, path::PathBuf, vec};
use std::io::BufReader;
use std::fs::{self, DirEntry};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::collections::HashMap;
use std::collections::HashSet;

fn sort_types_fields(types:&HashMap<String, u32>, fields:&HashMap<String, u32>) -> (Vec<String>,Vec<String>){
  // let mut types_vec: Vec<(String, u32)>;
  let mut types_vec: Vec<(String, u32)> = vec![];

  for (t,c) in types{
    types_vec.push((t.to_string(), *c));
  }  

  let mut ordered_types:Vec<String> = Vec::new();

  types_vec.sort_by(|a, b| a.1.cmp(&b.1).reverse());
  for (key, value) in types_vec {
    println!("{} {}", key, value);
    ordered_types.push(key);
  }

  println!("");

  let mut fields_vec: Vec<(String, u32)> = vec![];

  for (t,c) in fields{
    fields_vec.push((t.to_string(), *c));
  }  

  let mut ordered_fields:Vec<String> = Vec::new();

  fields_vec.sort_by(|a, b| a.1.cmp(&b.1).reverse());
  for (key, value) in fields_vec {
    println!("{} {}", key, value);
    ordered_fields.push(key);
  }
  (ordered_types, ordered_fields)
}

fn get_statistics(Entries:&Vec<Entry>) -> (HashMap<String, u32>, HashMap<String, u32> ){
  let mut types: HashMap<String, u32> = HashMap::new();
  let mut fields: HashMap<String, u32> = HashMap::new();

  for entry in Entries{
    if types.contains_key(&entry.Type){
      *types.get_mut(&entry.Type).unwrap() += 1;
    }else{
      types.insert(entry.Type.to_string(), 1);
    }
  
    for (field, _) in &entry.Fields_Values{
      if fields.contains_key(field){
        *fields.get_mut(field).unwrap() += 1;
      }else{
        fields.insert(field.to_string(), 1);
      }
    }
  }
  (types, fields)
}
#[derive(PartialEq)]
struct Author{
  first_name:String,
  last_name:String
}

#[derive(PartialEq)]
struct Entry {
  Type: String,
  Key: String,
  Authors:Vec<Author>,
  Files: Vec<String>,
  has_file:bool,
  Fields_Values: HashMap<String, String>,
  Tags:Vec<String>,
}

fn Entry_to_String_bib(e: &Entry) -> String{

  // type and key
  let mut s = format!("@{}{{{},\n",e.Type, e.Key);
  // authors
  let mut t = e.Authors.iter().map(|x| format!("{} {}", x.first_name, x.last_name)).collect::<Vec<String>>().join(" and ");
  s.push_str(&format!("author = {{{}}},\n", t));
  // Fields & Values
  t = e.Fields_Values.iter().map(|x| format!("{} = {{{}}},\n", x.0, x.1)).collect::<Vec<String>>().join("");
  s.push_str(&t);


  s.push_str("\n}");
  s
}

fn create_Entry(Type:String, Key:String) -> Entry{
  Entry{
    Type:Type, 
    Key: Key,
    Authors: vec![],
    Fields_Values: HashMap::new(),
    Files: vec![],
    Tags: vec![],
    has_file:false
  }
}

fn recursive_paths(path:&str, bib_paths:&mut Vec<PathBuf>, doc_paths:&mut Vec<PathBuf>){
  let exts=vec!["pdf","djvu,epub"];
  for entry in fs::read_dir(path).unwrap() {
    let entry = entry.unwrap();
    if entry.path().is_dir(){
      recursive_paths(&entry.path().to_str().unwrap().to_owned(), bib_paths, doc_paths);
    }
    else{
      if entry.path().extension().is_some(){
        let ext = entry.path().extension().unwrap().to_owned();
        // let ext = osext.to_str().unwrap()
        if ext == "bib" {
          bib_paths.push(entry.path());
        }
        else if exts.contains(&ext.to_str().unwrap()) {
          doc_paths.push(entry.path());
        }
      }
    }
  }
}

fn read_bib(path:PathBuf, bib_lines:&mut Vec<String>){
  let mut file = File::open(path).unwrap();
  let file_buffer = BufReader::new(file);

  let mut inside_comment=false;
  for line in file_buffer.lines(){
    let l =line.unwrap();
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

fn parse_file_field(original_value:&str) -> Vec<String>{

  let vec:Vec<String> = original_value.split(";").map(|s| s.to_owned()).collect();
  let mut out:Vec<String> = vec![];
  for v in &vec{
    // v = &v.trim_end_matches(":PDF").trim_end_matches(":application/pdf").to_string();
    let path = Path::new(&v.trim_end_matches(":PDF").trim_end_matches(":application/pdf").to_string()).to_owned();
    out.push(path.file_name().unwrap().to_str().unwrap().to_string());
  }
  out
}

fn parse_author_field(original_value:&str) -> Vec<Author>{
  let mut authors: Vec<Author> = vec![];
  for fl in original_value.split("and"){
    if fl.contains(","){
      let fl_vec: Vec<&str> = fl.split(",").collect();
      if fl_vec.len() == 2 {
        authors.push(Author{first_name:fl_vec[1].trim().to_string(), last_name:fl_vec[0].trim().to_string()})
      }
      else{
        authors.push(Author{first_name:" ".to_string(), last_name:fl_vec[0].trim().to_string()})

      }
    }
    else if fl.contains(" "){
      let fl_vec: Vec<&str> = fl.split(" ").collect();
      if fl_vec.len() == 2 {
        authors.push(Author{first_name:fl_vec[0].trim().to_string(), last_name:fl_vec[1].trim().to_string()})
      }
      else{
        authors.push(Author{first_name:" ".to_string(), last_name:fl_vec[0].trim().to_string()})
      }
    }
    else{
      authors.push(Author{first_name:" ".to_string(), last_name:fl.trim().to_string()})
    }
  }
  authors
}

fn parse_tags_field(original_value:&str) -> Vec<String>{
  let mut tags: Vec<String> = 
  original_value.replace(";", ",").split(",").map(|x| x.to_lowercase().trim().to_owned()).collect();
  tags
}

fn parse_bib(lines:&Vec<String> )->Vec<Entry>{
  let mut Entries : Vec<Entry> = vec![];

  let patterns : &[_] = &['{', '}','\t',',']; 
  let mut counter =0;
  while counter < lines.len() {
    if lines[counter].starts_with("@"){
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
      while counter < lines.len() && lines[counter].trim() != "}"{
        let mut field_value=String::new();
        while 
        counter < lines.len() - 1 && 
        !(lines[counter].trim().ends_with("}") && lines[counter+1].trim() == "}" ) && 
        !lines[counter].trim().ends_with("},") 
        {
          field_value.push_str(lines[counter].trim_matches('\n'));
          counter +=1;
        }
        field_value.push_str(lines[counter].trim_matches('\n'));
        
        let vec: Vec<&str> = field_value.splitn(2,"=").collect();
        if vec.len() == 2 {
          let field:&str=&vec[0].trim().trim_matches(patterns).to_lowercase();
          let mut value=vec[1].trim().trim_matches(patterns);
          let mut last_entry =Entries.last_mut().unwrap();

          match field {
            "file" => last_entry.Files = parse_file_field(value),
            "author" => last_entry.Authors = parse_author_field(value),
            "mendeley-tags" |"groups" | "keywords" => last_entry.Tags = parse_tags_field( value),
            _ => {
              if Entries.last().unwrap().Fields_Values.contains_key(field){
                println!("Repeated entry at {}\n", field_value);
              }
              else{
                Entries.last_mut().unwrap().Fields_Values.insert(field.to_string(), value.to_string());
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

fn write_csv(path: &str, entries: &Vec<Entry>, ordered_fields: &Vec<String>){
  let path = Path::new(path);
  let display = path.display();

  // Open a file in write-only mode, returns `io::Result<File>`
  let mut f = match File::create(&path) {
      Err(why) => panic!("couldn't create {}: {}", display, why),
      Ok(file) => file,
  };

  write!(f,"\u{feff}"); // BOM, indicating uft8 for excel

  // let fields_vec: Vec<> = ordered_fields.into_iter().map(|x| x).collect();
  let top_row=String::from("file?,type,key,author,") + (&ordered_fields.join(",")) + ",file";
  writeln!(f, "{}", top_row).unwrap();

  for e in entries{

    let c0 = match e.has_file {
      true => "x".to_string(),
      false=> " ".to_string()
    };

    let c1:String = e.Authors.iter().map(|a| a.first_name.to_owned() + " " + &a.last_name).collect::<Vec<String>>().join(" & ");
    let mut row:Vec<String> = vec![c0, e.Type.to_owned(), e.Key.to_owned(), c1];

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
    row.push(format!("\"{}\"",e.Files.join(",")));
    writeln!(f, "{}",row.join(",")).unwrap();
  }
}

fn write_bib(path: &str, entries: & Vec<Entry>){
  let path = Path::new(path);
  let display = path.display();

  // Open a file in write-only mode, returns `io::Result<File>`
  let mut f = match File::create(&path) {
      Err(why) => panic!("couldn't create {}: {}", display, why),
      Ok(file) => file,
  };

  for e in entries{
    writeln!(f, "{}", Entry_to_String_bib(&e)).unwrap();
  }
}

fn paths_to_filenames(paths:&Vec<PathBuf>)->Vec<String>{
  let mut filenames:Vec<String> = vec![];
  for p in paths{
    filenames.push(p.file_name().unwrap().to_str().unwrap().to_string());
    // println!("{}", filenames.last().unwrap());
  }
  filenames
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

fn main() {
  let path="bibs";

  let mut bib_paths= vec![];
  let mut doc_paths= vec![];
  recursive_paths(path, &mut bib_paths, &mut doc_paths);
  let path="C:/Users/tesse/Desktop/Files/Dropbox/BIBrep/";
  recursive_paths(path, &mut bib_paths, &mut doc_paths);

  
  let mut bib_vec = vec![];
  for p in bib_paths {
    println!("{:?}", p);
    read_bib(p, &mut bib_vec);
  }
  write_raw_bib("Complete.txt", &mut bib_vec);

  let mut Entries = parse_bib(&bib_vec);

  // let (types, fields) = get_statistics(&Entries);
  // let (ordered_types, ordered_fields) = sort_types_fields(&types, &fields);

  remove_identical_Entries(& mut Entries);

  let p = paths_to_filenames(&doc_paths.to_owned());
  check_files(& mut Entries, &p);

  let (types, fields) = get_statistics(&Entries);
  let (ordered_types, ordered_fields) = sort_types_fields(&types, &fields);

  write_bib("Complete.bib", &Entries);
  write_csv("Complete.csv", &Entries, &ordered_fields);
}

fn check_files(entries: & mut Vec<Entry>, doc_paths: & Vec<String>){
  for e in entries{
    for f in e.Files.to_owned(){
      if doc_paths.contains(&f){
        e.has_file = true;
      }
      else if e.Authors.len()>0 && e.Fields_Values.contains_key("year") && e.Fields_Values.contains_key("title"){
        let t = format!(
          "{} {} - {}{}.pdf",
          e.Fields_Values["year"],
          e.Fields_Values["title"],
          &e.Authors[0].last_name,
          if e.Authors.len() > 1 { " et al" } else { "" }
        );
        // println!("{}", t);
          if doc_paths.contains(&t){
            e.has_file = true;
            println!("{}", t);
          }
      }
    }
  }
}

fn remove_identical_Entries(mut entries: & mut Vec<Entry>){
  // Remove identical entries
  let mut repeated: Vec<usize> = vec![];
  for i in 0..entries.len(){
    for j in i+1..entries.len(){
      if entries[i] == entries[j] {
        // println!("{}", entries[i].Fields_Values["title"]);
        // println!("{}", entries[j].Fields_Values["title"]);
        repeated.push(j);
      }
    }
  }  
  println!("Identical {}", &repeated.len());

  repeated.sort();
  repeated.reverse();
  for i in repeated{
    entries.remove(i);
  }

  // Remove entries were only key and/or type are different
  let mut repeated: Vec<usize> = vec![];
  for i in 0..entries.len(){
    for j in i+1..entries.len(){
      if 
      entries[i].Fields_Values.contains_key("title") &&
      entries[j].Fields_Values.contains_key("title") &&
      entries[i].Fields_Values["title"] == entries[j].Fields_Values["title"]
      // entries[i].Fields_Values["title"].to_ascii_lowercase() == entries[j].Fields_Values["title"].to_ascii_lowercase()
      {
        if entries[i].Fields_Values == entries[j].Fields_Values{
          // println!("{}", entries[i].Fields_Values["title"]);
          // println!("{}", entries[j].Fields_Values["title"]);
          repeated.push(j);
          for file in entries[j].Files.to_owned(){
            if !entries[i].Files.contains(&file){
              entries[i].Files.push(file.to_string());
            }
          }
        }
      }
    }
  }
  println!("Differences in key and/or type {}", &repeated.len());

  repeated.sort();
  repeated.reverse();
  for i in repeated{
    entries.remove(i);
  }
  
  remove_by_field( entries, "doi");// Check entries with same doi
  remove_by_field( entries, "issn"); // Check entries with same issn
  remove_by_field( entries, "isbn");// Check entries with same isbn
  remove_by_field( entries, "url");// Check entries with same url
  remove_by_field( entries, "shorttitle");// Check entries with same shorttitle
  remove_by_field( entries, "pmid");// Check entries with same pmid
  remove_by_field( entries, "abstract");// Check entries with same abstract
  remove_by_field( entries, "eprint");// Check entries with same eprint
  remove_by_field( entries, "arxivid");// Check entries with same arxivid




  println!("");
}

fn remove_by_field(mut entries: & mut Vec<Entry>, field:&str){
  // Check entries with same doi
  let mut repeated: Vec<usize> = vec![];
  for i in 0..entries.len(){
    for j in i+1..entries.len(){
      if 
      entries[i].Fields_Values.contains_key(field) &&
      entries[j].Fields_Values.contains_key(field) &&
      entries[i].Fields_Values[field] == entries[j].Fields_Values[field]
      {
        let merged = merge(&mut entries, i, j);
        if merged{
          repeated.push(j);
        }
      }
    }
  }

  println!("Same {}, compatible {}", field, &repeated.len());
  repeated.sort();
  repeated.reverse();
  for i in repeated{
    entries.remove(i);
  }
}

fn merge(entries: &mut Vec<Entry>, i: usize, j: usize) -> bool{
  // let e1 = &mut entries[i];
  // let e2 = &entries[j];
  let f1:HashSet<String> = entries[i].Fields_Values.iter().map(|x| x.0.to_owned()).collect();
  let f2:HashSet<String> = entries[j].Fields_Values.iter().map(|x| x.0.to_owned()).collect();
  let intersection = f1.intersection(&f2).to_owned();
  let common_fields:Vec<&String> = intersection.collect();
  let mut eq = true;
  for field in common_fields{
    if entries[i].Fields_Values[field] != entries[j].Fields_Values[field]{
      eq = false;
      break;
    }
  }
  if eq{
    // println!("{}", entries[i].Fields_Values["title"]);
    // println!("{}\n", entries[j].Fields_Values["title"]);
    let mut files_to_add:Vec<String> = vec![];
    for file in &entries[j].Files{
      if !&entries[i].Files.contains(file){
        files_to_add.push(file.to_string());
      }
    }
    entries[i].Files.append(&mut files_to_add);


    for field in f2.difference(&f1){
      let value =entries[j].Fields_Values.get_mut(field).unwrap().to_string();
      if let Some(x) = entries[i].Fields_Values.get_mut(field) {
        *x = value;
      }
    }
  }
  eq
}