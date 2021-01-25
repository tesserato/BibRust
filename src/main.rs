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

use std::io::BufReader;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::collections::HashSet;
use std::str;
// use itertools::Itertools;
// use alloc::collections;

static INTERNAL_TAG_MARKER: char ='#';
static REVIEWED: &str = "#reviewed";

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

    if entry.Authors.len() > 0{
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
  Files: HashSet<String>,
  Fields_Values: HashMap<String, String>,
  Tags:HashSet<String>,
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
    t = e.Files.iter().map(|x| x.replace(":", "\\:")).collect::<Vec<String>>().join(";");
    s.push_str(&format!("file = {{{}}},\n", t));
  }

  //Tags
  if !e.Tags.is_empty() {
    t = e.Tags.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(",");
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
    Files: HashSet::new(),
    Tags: HashSet::new(),
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

fn parse_tags_field(e: &mut Entry, original_value:&str) -> HashSet<String>{
  // let patterns : &[_] = &['{', '}','\t',',',' ',','];
  let mut tags: HashSet<String> = 
    original_value
      .replace(";", ",")
      .split(",").map(|x| x.to_lowercase().trim_matches(|c:char| c != INTERNAL_TAG_MARKER && !c.is_alphabetic()).to_owned())
      .filter(|s| !s.is_empty())
      .collect();
  tags
}

fn parse_file_field(e: &mut Entry, value:&String){
  let mut checked: HashSet<String> = HashSet::new();
  for raw_f in value.split(";"){
    let paths = raw_f
      .split(":")
      .filter(|x| !x.is_empty() && x.contains("."))
      .map(|x| format!("C:{}", x.replace("\\\\", "/").replace("\\", "/")))
      .collect::<Vec<String>>();
      for p in paths{
        if Path::new(&p).exists(){
          // println!("{}", p);
          checked.insert(Path::new(&p).as_os_str().to_str().unwrap().to_string());
        }
        else{
          if e.Fields_Values.contains_key("broken-files"){
            e.Fields_Values.get_mut("broken-files").unwrap().push_str(&format!(",{}", p).to_string());
          }
          else{
            e.Fields_Values.insert("broken-files".to_string(), p);
          }
        }
      }
    }
  e.Files = checked;
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
          let field:&str = &vec[0].trim().trim_matches(patterns).to_lowercase();
          let mut value = vec[1];
          let mut last_entry = Entries.last_mut().unwrap();
          match field {
            "file" => parse_file_field(last_entry, &value.to_string()), //parse_file_field(value),
            "author" => last_entry.Authors = parse_author_field(value),
            "mendeley-tags"|"groups"|"tags" => last_entry.Tags = parse_tags_field( last_entry, value),
            _ => {
              if Entries.last().unwrap().Fields_Values.contains_key(field){
                println!("Repeated entry at {}\n", field_value);
              }
              else{
                Entries.last_mut().unwrap().Fields_Values.insert(field.to_string(), parse_generic_field(value));
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

  let top_row=String::from("reviewed,entry type,key,author,") + (&ordered_fields.join(",")) + ",tags,file";
  writeln!(f, "{}", top_row).unwrap();

  for e in entries{
    let c0:String = match e.Tags.contains(REVIEWED){
      true => "x".to_string(),
      false => "".to_string(),
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
    row.push(e.Tags.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(";"));
    row.push(format!("\"{}\"", e.Files.iter().cloned().collect::<Vec<String>>().join(";")));
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

fn write_to_ads(){
  let path = Path::new("ads_test.txt:ads.bib");
  let display = path.display();

  // Open a file in write-only mode, returns `io::Result<File>`
  let mut f = match File::create(&path) {
      Err(why) => panic!("couldn't create {}: {}", display, why),
      Ok(file) => file,
  };


  writeln!(f, "test 01\ntest 02");
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

  get_files_from_paths(&mut main_entries, &doc_paths);

  let mut other_entries = get_entries_from_root_path("bibs/other".to_string());
  remove_redundant_Entries(&mut other_entries);

  get_files_from_entries(&mut main_entries, &other_entries);

  inspect_entries(&mut main_entries);
  let (_, ordered_fields) = get_statistics(&main_entries);

  write_bib("Complete.bib", &main_entries);
  write_csv("Complete.csv", &main_entries, &ordered_fields);
}

fn inspect_entries(entries: &mut Vec<Entry>){
  for e in entries{
    // check author field
    if e.Authors.is_empty() {
      e.Tags.insert("#no author".to_string());
    }
    else if !e.Authors
              .iter()
              .map(|x| format!("{} {}", x.first_name, x.last_name))
              .collect::<Vec<String>>()
              .join("")
              .replace(".", "")
              .replace(" ", "")
              .chars().all(char::is_alphanumeric)
    {
      e.Tags.insert("#corrupted author".to_string());
    }
  }
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
  println!("Removed {} Identical entries\n", &repeated.len());
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

    let files = entries[j].Files.iter().cloned().collect::<Vec<String>>();
    for file in files{
      if !entries[i].Files.contains(&file){
        entries[i].Files.insert(file.to_string());
      }
    }

    for field in f2.difference(&f1){
      let value =entries[j].Fields_Values.get_mut(field).unwrap().to_string();
      if let Some(x) = entries[i].Fields_Values.get_mut(field) {
        *x = value;
      }
    }
  }
  eq
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

// fn sort_types_fields(types:&HashMap<String, u32>, fields:&HashMap<String, u32>) -> (Vec<String>,Vec<String>){
//   // let mut types_vec: Vec<(String, u32)>;
//   let mut types_vec: Vec<(String, u32)> = vec![];

//   for (t,c) in types{
//     types_vec.push((t.to_string(), *c));
//   }  

//   let mut ordered_types:Vec<String> = Vec::new();

//   types_vec.sort_by(|a, b| a.1.cmp(&b.1).reverse());
//   for (key, value) in types_vec {
//     println!("{} {}", key, value);
//     ordered_types.push(key);
//   }

//   println!("");

//   let mut fields_vec: Vec<(String, u32)> = vec![];

//   for (t,c) in fields{
//     fields_vec.push((t.to_string(), *c));
//   }  

//   let mut ordered_fields:Vec<String> = Vec::new();

//   fields_vec.sort_by(|a, b| a.1.cmp(&b.1).reverse());
//   for (key, value) in fields_vec {
//     println!("{} {}", key, value);
//     ordered_fields.push(key);
//   }
//   (ordered_types, ordered_fields)
// }
