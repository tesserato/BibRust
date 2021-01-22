#![feature(str_split_once)]
#![allow(non_snake_case)]

// extern crate pdf_extract;
// extern crate lopdf;
// use pdf_extract::*;
// use lopdf::*;
// use std::path;
// use std::env;
// let path_to_pdf = path::Path::new("biblatex.pdf");
// let result = extract_text(path_to_pdf);

use std::{char::ToLowercase, hint::unreachable_unchecked, io, iter::{Map, Rev}, path::{self, PathBuf}, vec};
use std::io::BufReader;
use std::fs::{self, DirEntry};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::collections::HashMap;
use std::collections::HashSet;
use std::str;


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

// #[derive(PartialEq)]
struct Entry {
  Type: String,
  Key: String,
  Authors:Vec<Author>,
  Files: Vec<String>,
  has_file:bool,
  Fields_Values: HashMap<String, String>,
  Tags:Vec<String>,
}

impl PartialEq for Entry {
  fn eq(&self, other: &Self) -> bool {
      self.Type == other.Type &&
      self.Authors == other.Authors &&
      self.Fields_Values == other.Fields_Values &&
      self.Tags == other.Tags &&
      self.Files == other.Files
  }
}

fn Entry_to_String_bib(e: & Entry) -> String{

  // type and key
  let mut s = format!("@{}{{{},\n",e.Type, e.Key);
  let mut t="".to_string();

  // authors
  if e.Authors.len() > 0 {
    t = e.Authors.iter().map(|x| format!("{} {}", x.first_name, x.last_name)).collect::<Vec<String>>().join(" and ");
    s.push_str(&format!("author = {{{}}},\n", t));
  }

  // Fields & Values
  if e.Fields_Values.len() > 0 {
    t = e.Fields_Values.iter().map(|x| format!("{} = {{{}}},\n", x.0, x.1)).collect::<Vec<String>>().join("");
    s.push_str(&t);
  }

  //Files
  if e.Files.len() > 0 {
    t = e.Files.iter().map(|x| x.replace(":", "\\:")).collect::<Vec<String>>().join(",");
    s.push_str(&format!("file = {{{}}},\n", t));
  }

  //Tags
  if e.Tags.len() > 0 {
    let mut tags = e.Tags.to_owned();
    tags.sort();
    t = tags.join(",");
    s.push_str(&format!("keywords = {{{}}},\n", t));
    s.push_str(&format!("mendeley-tags = {{{}}},\n", t));
  }
  s.push_str("}\n");
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

fn read_bib(path:PathBuf, bib_lines:&mut Vec<String>){
  let mut file = File::open(path).unwrap();
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

// fn parse_file_field(original_value:&str) -> Vec<String>{
//   let vec:Vec<String> = original_value.split(";").map(|s| s.to_owned()).collect();
//   let mut out:Vec<String> = vec![];
//   for v in &vec{
//     let clean = v
//       .trim_end_matches(":PDF")
//       .trim_end_matches(":pdf")
//       .trim_end_matches(":application/pdf")
//       .trim_matches(':')
//       .replace("\\:", ":")
//       .replace("\\", "/");
//       println!("{}",clean);
//       let path =Path::new(&clean);
//     if path.exists(){
//       out.push(path.as_os_str().to_str().unwrap().to_string());
//     }
//   }
//   out
// }

fn parse_author_field(original_value:&str) -> Vec<Author>{
  let mut authors: Vec<Author> = vec![];
  let patterns : &[_] = &['{', '}','\t',',',' '];
  for fl in original_value.split("and").map(|x| x.trim()){
    if fl.contains(","){
      let fl_vec = fl.rsplit_once(",").unwrap();
      authors.push(Author{
        first_name:fl_vec.1.trim().trim_matches(patterns).to_string(), 
        last_name:fl_vec.0.trim().trim_matches(patterns).to_string()
      })
    }
    else if fl.contains(" "){
      let fl_vec = fl.rsplit_once(" ").unwrap();
      authors.push(Author{
        first_name:fl_vec.0.trim().trim_matches(patterns).to_string(), 
        last_name:fl_vec.1.trim().trim_matches(patterns).to_string()
      })
    }
    else{
      authors.push(Author{
        first_name:"".to_string(), 
      last_name:fl.trim().trim_matches(patterns).to_string()
    })
    }
  }
  authors
}

fn parse_tags_field(original_value:&str) -> Vec<String>{
  // let patterns : &[_] = &['{', '}','\t',',',' ',','];
  let mut tags: Vec<String> = 
    original_value
      .replace(";", ",")
      .split(",").map(|x| x.to_lowercase().trim_matches(|c:char| !c.is_alphabetic()).to_owned())
      .filter(|s| !s.is_empty())
      .collect();
  tags
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
  let mut counter =0;
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
        let mut field_value=String::new();
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
          let field:&str=&vec[0].trim().trim_matches(patterns).to_lowercase();
          let mut value = &parse_generic_field(vec[1]);
          let mut last_entry = Entries.last_mut().unwrap();

          match field {
            "file" => last_entry.Files = vec![value.to_string()], //parse_file_field(value),
            "author" => last_entry.Authors = parse_author_field(value),
            "mendeley-tags" |"groups" => last_entry.Tags = parse_tags_field( value),
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

fn find_paths_to_files_with_ext(root_path:&str, paths:&mut Vec<PathBuf>, exts:& Vec<String>){  
  for dir_entry in fs::read_dir(root_path).unwrap() {
    let p = dir_entry.unwrap().path();
    if p.is_dir(){
      find_paths_to_files_with_ext(&p.to_str().unwrap().to_owned(), paths, &exts);
    }
    else if
    p.extension().is_some() 
    && exts.contains(&p.extension().unwrap().to_str().unwrap().to_lowercase().to_owned())
    {
      paths.push(p);
    }
  }
}

fn get_entries_from_root_path(root_path:String) -> Vec<Entry>{
  let exts=vec!["bib".to_string()];
  let mut bib_paths= vec![];
  find_paths_to_files_with_ext(&root_path, &mut bib_paths, &exts);

  let mut bib_vec = vec![];
  for p in bib_paths {
    println!("{:?}", p);
    read_bib(p, &mut bib_vec);
  }
  write_raw_bib("Complete_raw.bib", &mut bib_vec);

  parse_bib(&bib_vec)
}

fn main() {

  let mut main_entries = get_entries_from_root_path("bibs/main".to_string());
  remove_redundant_Entries(& mut main_entries);

  let mut doc_paths: Vec<PathBuf> = vec![];
  find_paths_to_files_with_ext(
    "C:/Users/tesse/Desktop/Files/Dropbox/BIBrep",
    &mut doc_paths,
    &vec!["pdf".to_string(),"epub".to_string(),"djvu".to_string()]
  );

  check_files(&mut main_entries, &doc_paths);

  let (types, fields) = get_statistics(&main_entries);
  let (_, ordered_fields) = sort_types_fields(&types, &fields);

  write_bib("Complete.bib", &main_entries);
  write_csv("Complete.csv", &main_entries, &ordered_fields);
}

fn check_files(entries: & mut Vec<Entry>, doc_paths: &Vec<PathBuf>){

  let mut filename_path: HashMap<String, PathBuf> = HashMap::new();
  for p in doc_paths{
    filename_path.insert(
      p.file_name().unwrap().to_str().unwrap().to_string(), 
      p.to_path_buf()
    );
  }

  for e in entries{
    let mut checked: Vec<String> = vec![];
    if e.Files.len() > 0 {
      for raw_f in e.Files.to_owned(){
        let paths = raw_f
          .split(":")
          .filter(|x| !x.is_empty() && x.contains("."))
          .map(|x| format!("C:{}", x.replace("\\\\", "/").replace("\\", "/")))
          .collect::<Vec<String>>();
          for p in paths{
            if Path::new(&p).exists(){
              // println!("{}", p);
              e.has_file = true;
              checked.push(Path::new(&p).as_os_str().to_str().unwrap().to_string())
            }
            else{
              let filename = p.split("/").last().unwrap();
              if filename_path.contains_key(filename){
                // println!("{}", filename);
                e.has_file = true;
                checked.push(filename_path[filename].as_path().to_str().unwrap().to_string())
  
              }
            }
          }
    }

      // else if e.Authors.len()>0 && e.Fields_Values.contains_key("year") && e.Fields_Values.contains_key("title"){
      //   let t = format!(
      //     "{} {} - {}{}.pdf",
      //     e.Fields_Values["year"],
      //     e.Fields_Values["title"],
      //     &e.Authors[0].last_name,
      //     if e.Authors.len() > 1 { " et al" } else { "" }
      //   );
      //   // println!("{}", t);
      //     if doc_paths.contains(&t){
      //       e.has_file = true;
      //       println!("{}", t);
      //     }
      // }
    }
    e.Files = checked;
  }
}

fn remove_redundant_Entries(mut entries: & mut Vec<Entry>){
  // Remove identical entries
  let mut repeated: Vec<usize> = vec![];
  for i in 0..entries.len(){
    for j in i+1..entries.len(){
      if entries[i] == entries[j] {
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

  // // Remove entries were only key and/or type are different
  // let mut repeated: Vec<usize> = vec![];
  // for i in 0..entries.len(){
  //   for j in i+1..entries.len(){
  //     if 
  //     entries[i].Fields_Values.contains_key("title") &&
  //     entries[j].Fields_Values.contains_key("title") &&
  //     entries[i].Fields_Values["title"] == entries[j].Fields_Values["title"]
  //     // entries[i].Fields_Values["title"].to_ascii_lowercase() == entries[j].Fields_Values["title"].to_ascii_lowercase()
  //     {
  //       if entries[i].Fields_Values == entries[j].Fields_Values{
  //         // println!("{}", entries[i].Fields_Values["title"]);
  //         // println!("{}", entries[j].Fields_Values["title"]);
  //         repeated.push(j);
  //         for file in entries[j].Files.to_owned(){
  //           if !entries[i].Files.contains(&file){
  //             entries[i].Files.push(file.to_string());
  //           }
  //         }
  //       }
  //     }
  //   }
  // }
  // println!("Differences in key and/or type {}", &repeated.len());
  // repeated.sort();
  // repeated.reverse();
  // for i in repeated{
  //   entries.remove(i);
  // }
  
  // remove_by_field( entries, "doi");// Check entries with same doi
  // remove_by_field( entries, "issn"); // Check entries with same issn
  // remove_by_field( entries, "isbn");// Check entries with same isbn
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
    if entries[i].Fields_Values[field].to_lowercase() != entries[j].Fields_Values[field].to_lowercase(){
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


// fn recursive_paths(path:&str, bib_paths:&mut Vec<PathBuf>, doc_paths:&mut Vec<PathBuf>){
//   let exts=vec!["pdf","djvu,epub"];
//   for entry in fs::read_dir(path).unwrap() {
//     let entry = entry.unwrap();
//     if entry.path().is_dir(){
//       recursive_paths(&entry.path().to_str().unwrap().to_owned(), bib_paths, doc_paths);
//     }
//     else{
//       if entry.path().extension().is_some(){
//         let ext = entry.path().extension().unwrap().to_owned();
//         // let ext = osext.to_str().unwrap()
//         if ext == "bib" {
//           bib_paths.push(entry.path());
//         }
//         else if exts.contains(&ext.to_str().unwrap()) {
//           doc_paths.push(entry.path());
//         }
//       }
//     }
//   }
// }

// fn paths_to_filenames(paths:&Vec<PathBuf>)->Vec<String>{
//   let mut filenames:Vec<String> = vec![];
//   for p in paths{
//     filenames.push(p.file_name().unwrap().to_str().unwrap().to_string());
//     // println!("{}", filenames.last().unwrap());
//   }
//   filenames
// }