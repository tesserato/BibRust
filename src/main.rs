use std::{io, path::PathBuf};
use std::io::{BufReader};
use std::fs::{self, DirEntry};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::collections::HashMap;

struct Entry {
  Type: String,
  Key: String,
  Fields_Values: HashMap<String, String>,
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
      bib_lines.push(l.to_string());
    }
    if inside_comment && l.ends_with('}'){
      inside_comment = false;
    }

  }
}

fn parse_bib(lines:&Vec<String> )->(Vec<Entry>, HashMap<String, i32>,HashMap<String, i32>){
  let mut Entries : Vec<Entry> = vec![];
  let mut types =HashMap::new();
  let mut fields =HashMap::new();

  let patterns : &[_] = &['{', '}','\t',',']; 
  let mut counter =0;
  while counter < lines.len() {
    if lines[counter].starts_with("@"){
      let vec: Vec<&str> = lines[counter].splitn(2,"{").collect();
      if vec.len() == 2 {
        let Type =vec[0].trim().trim_matches('@').to_lowercase();
        let Key =vec[1].trim().trim_matches(',');
        if types.contains_key(&Type){
          *types.get_mut(&Type).unwrap() += 1;
        }else{
          types.insert(Type.to_string(), 1);
        }
        Entries.push(Entry{Type:Type, Key: Key.to_string(), Fields_Values: HashMap::new()}) ;
      }
      else{
        println!("\n{}\n",lines[counter]);
      }
      counter +=1;
      while counter < lines.len() && lines[counter].trim() != "}"{


        let mut field_value=String::new();
        while counter < lines.len() - 1 && !(lines[counter].trim().ends_with("}") && lines[counter+1].trim() == "}" ) && !lines[counter].trim().ends_with("},") {
          field_value.push_str(lines[counter].trim_matches('\n'));
          counter +=1;
        }
        field_value.push_str(lines[counter].trim_matches('\n'));
        let vec: Vec<&str> = field_value.splitn(2,"=").collect();
        if vec.len() == 2 {
            let field=vec[0].trim().trim_matches(patterns);
            let value=vec[1].trim().trim_matches(patterns);

            if fields.contains_key(field){
              *fields.get_mut(field).unwrap() += 1;
            }else{
              fields.insert(field.to_string(), 1);
            }

            if Entries.last().unwrap().Fields_Values.contains_key(field){
              println!("Repeated entry at {}\n", field_value);
            }
            else{
              Entries.last_mut().unwrap().Fields_Values.insert(field.to_string(), value.to_string());
            }
        }
        else{
          println!("Split vector with size != 2 at {}\n", field_value);
        }
        counter +=1
      }
    }
    counter +=1;
  }
  (Entries, types, fields)
}

fn write_csv(path: &str, entries: &Vec<Entry>, fields: &Vec<&String>){
  let path = Path::new(path);
  let display = path.display();

  // Open a file in write-only mode, returns `io::Result<File>`
  let mut f = match File::create(&path) {
      Err(why) => panic!("couldn't create {}: {}", display, why),
      Ok(file) => file,
  };

  write!(f,"\u{feff}"); // BOM, indicating uft8 for excel

  let fields:Vec<String> =fields.to_owned().into_iter().map(|x| x.to_owned()).collect();
  let top_row=String::from("type,key,") + (&fields.join(","));  
  writeln!(f, "{}", top_row).unwrap();


  for e in entries{
    let mut row:Vec<String> = vec![e.Type.to_owned(), e.Key.to_owned()];

    for field in &fields{
      if e.Fields_Values.contains_key(field){
        row.push(
          format!("\"{}\"", e.Fields_Values[field].to_owned())
        );
      }
      else{
        row.push(" ".to_string());
      }
    }
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
    writeln!(f, "@{}{{{},",e.Type, e.Key).unwrap();
    for (field,value) in &e.Fields_Values{
      writeln!(f, "{}={{{}}},",field, value).unwrap();
    }
    writeln!(f, "}}").unwrap();
    writeln!(f, "").unwrap();
  }
}

fn paths_to_filenames(paths:&Vec<PathBuf>)->Vec<&str>{
  let mut filenames:Vec<&str> = vec![];
  for p in paths{
    filenames.push(p.file_name().unwrap().to_str().unwrap());
    println!("{}", filenames.last().unwrap());
  }
  filenames
}

fn main() {
  let path="C:/Users/tesse/Desktop/Files/Dropbox/BIBrep/";

  let mut bib_paths= vec![];
  let mut doc_paths= vec![];
  recursive_paths(path, &mut bib_paths, &mut doc_paths);

  paths_to_filenames(&bib_paths);

  let mut bib_vec = vec![];
  for p in bib_paths {
    println!("{:?}", p);
    read_bib(p, &mut bib_vec);
  }


  let (Entries, types, fields) = parse_bib(&bib_vec);

  let mut types_vec: Vec<_> = types.iter().collect();
  types_vec.sort_by(|a, b| a.1.cmp(b.1).reverse());
  for (key, value) in types_vec {
    println!("{} {}", key, value);
  }

  println!("{}"," ");
  let mut fields_vec: Vec<_> = fields.iter().collect();
  let mut ordered_fields:Vec<&String> = Vec::new();

  fields_vec.sort_by(|a, b| a.1.cmp(b.1).reverse());
  for (key, value) in fields_vec {
    println!("{} {}", key, value);
    ordered_fields.push(key);
  }


  // :Vec<String> = fields_vec.into_iter().map(|x| x.0).collect();

  write_bib("Complete.bib", &Entries);
  write_csv("Complete.csv", &Entries, &ordered_fields);
}


// fn write_file(path: &str, content: &Vec<String>){
//   let path = Path::new(path);
//   let display = path.display();

//   // Open a file in write-only mode, returns `io::Result<File>`
//   let mut file = match File::create(&path) {
//       Err(why) => panic!("couldn't create {}: {}", display, why),
//       Ok(file) => file,
//   };

//   for line in content{
//     writeln!(&mut file, "{}",line).unwrap();
//   }
// }