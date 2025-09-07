use colored::*;
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::Read;
use std::io::{self, BufReader, BufWriter, Write};
use std::path::Path;
use std::{env, process};

pub struct Entry{
    pub todo_entry: String,
    pub done: bool,
}

impl Entry {
    pub fn new(todo_entry: String, done: bool) -> Self {
        Self {
            todo_entry,
            done,
        }
    }
pub fn file_line(&self) -> String{
        let symbol = if self.done { "[*] " } else { "[ ]" };
        format!("{}{}\n", symbol, self.todo_entry)
    }

    pub fn list_line(&self, number: usize) -> String{
        //check if current task is completed or not 
        let todo_entry = if self.done {
            //Done, print it 
            self.todo_entry.strikethrough().to_string()
        } else {
            // !Done, print task 
            self.todo_entry.clone() 
        };
        format!("{number} {todo_entry}\n")
    }

    pub fn read_line(line: &String) -> Self{
        let done = &line[..4] == "[*] ";
        let todo_entry = (&line[4..]).to_string();
        Self { 
            todo_entry,
            done,
        }
    }

    pub fn raw_line(&self) -> String {
format!("{}\n", self.todo_entry)
    }
}

pub struct Todo {
    pub todo: Vec<String>, //re-sizable array 
    pub todo_path: String,
    pub todo_bak: String,
    pub no_backup: bool,
}

impl Todo {
//Result<> - "this might fail"
    pub fn new() -> Result<Self, String> {
        let todo_path: String = match env::var("TODO_PATH") {
            Ok(t) => t,
            Err(_) => {
                let home = env::var("HOME").unwrap();
                //unwrap only accessible in Result,Option 
                let legacy_todo = format!("{}/TODO", &home);
                match Path::new(&legacy_todo).exists(){
                    true => legacy_todo,
                    false => format!("{}/.todo", &home), 
                }
            }
        };
        let todo_bak: String = match env::var("TODO_BAK_DIR"){
            Ok(t) => t,
            Err(_) => String::from("/tmp/todo.bak"),
        };
        let no_backup = env::var("TODO_NOBACKUP").is_ok();
        let todofile = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(&todo_path)
            .expect("Couldn't open the todofile");

        //creates a new buf reader 
        let mut buf_reader = BufReader::new(&todofile);
        //empty string ready to be filled with todos 
        let mut contents = String::new();

        buf_reader.read_to_string(&mut contents).unwrap();
        let todo = contents.lines().map(str::to_string).collect();

        Ok(Self {
            todo,
            todo_path,
            todo_bak,
            no_backup,
        })
    }

    pub fn list(&self){
        let stdout = io::stdout();
        //buffered writer for stdout stream 
        let mut writer = BufWriter::new(stdout);
        let mut data = String::new();

        for(number, task) in self.todo.iter().enumerate(){
            let entry = Entry::read_line(task);
            let number = number + 1;

            let line = entry.list_line(number);
            data.push_str(&line);
        }
        writer
            .write_all(data.as_bytes())
            .expect("Failed to write to stdout");
    }

    pub fn raw(&self, arg: &[String]){
        if arg.len() > 1 {
            eprintln!("todo raw takes only 1 argument, not {}", arg.len())
        } else if arg.is_empty(){
            eprintln!("todo raw takes 1 argument (done/todo"); 
        } else {
            let stdout = io::stdout();
            let mut writer = BufWriter::new(stdout);
            let mut data = String::new();
            let arg = &arg[0];
            
            for task in self.todo.iter(){
                let entry = Entry::read_line(task);
                if entry.done && arg == "done" {
                    data = entry.raw_line();
                } else if !entry.done && arg == "todo" {
                    data = entry.raw_line();
                }
                writer 
                    .write_all(data.as_bytes())
                    .expect("Failed to write to stdout");
                }
        }
    }

    pub fn add(&self, args: &[String]) {
        if args.is_empty(){
            eprintln!("todo add takes at least 1 argument");
            process::exit(1);
        }

        let todofile = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.todo_path)
            .expect("Couldn't open the todofile");

        let mut buffer = BufWriter::new(todofile);
        for arg in args {
            if arg.trim().is_empty() {
                continue;
            }

            let entry = Entry::new(arg.to_string(), false);
            let line = entry.file_line();
            buffer
                .write_all(line.as_bytes())
                .expect("unable to write data");
            }
    }

    pub fn remove(&self, args: &[String]){
        if args.is_empty(){
            eprintln!("todo rm takes at least 1 argument");
            process::exit(1);
        }
        let todofile = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.todo_path)
            .expect("Couldn't open the todo file");

        let mut buffer = BufWriter::new(todofile);

        for (pos,line) in self.todo.iter().enumerate(){
            if args.contains(&(pos + 1).to_string()){
                continue;
            }
            let line = format!("{}\n", line);

            buffer 
                .write_all(line.as_bytes())
                .expect("unable to write data");
            }
    }

    fn remove_file(&self){
        match fs::remove_file(&self.todo_path){
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error while clearing todo file: {}", e)
            }
        };
    }

    pub fn reset(&self){
        if !self.no_backup {
            match fs::copy(&self.todo_path, &self.todo_bak){
                Ok(_) => self.remove_file(),
                Err(_) => {
                    eprintln!("Couldn't backup the todo file");
                }
            }
        } else {
            self.remove_file();
        }
    }
    pub fn restore(&self){
        fs::copy(&self.todo_bak, &self.todo_path).expect("unable to restore the backup");
    }

    pub fn sort(&self){
        let newtodo: String;
        let mut todo = String::new();
        let mut done = String::new();

        for line in self.todo.iter(){
            let entry = Entry::read_line(line);
            if entry.done {
                let line = format!("{}\n", line);
                done.push_str(&line);
            } else{
                let line = format!("{}\n", line);
                todo.push_str(&line);
            }
        }

        newtodo = format!("{}{}", &todo, &done);
        
        let mut todofile = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.todo_path)
            .expect("Couldn't open the todo file");

        todofile 
            .write_all(newtodo.as_bytes())
            .expect("Error while trying to save the todofile");
        }

    pub fn done(&self, args: &[String]){
        if args.is_empty(){
            eprintln!("todo done takes at least 1 argument");
            process::exit(1);
        }

        let todofile = OpenOptions::new()
            .truncate(true)
            .create(true)
            .write(true)
            .open(&self.todo_path)
            .expect("Couldn't open the todofile");
        let mut buffer = BufWriter::new(todofile);
        let mut data = String::new();
        
        for (pos,line) in self.todo.iter().enumerate(){
            let mut entry = Entry::read_line(line);
            let line = if args.contains(&(pos + 1).to_string()){
                entry.done = !entry.done;
                entry.file_line()
            } else {
                format!("{}\n", line)
            };
            data.push_str(&line);
        }
        buffer 
            .write_all(data.as_bytes())
            .expect("unable to write data");
        }

    pub fn edit(&self, args: &[String]){
        if args.is_empty() || args.len() != 2{
            eprintln!("todo edit takes exact 2 arguments");
            process::exit(1);
        }

        let todofile = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.todo_path)
            .expect("Couldn't open the todofile");
        let mut buffer = BufWriter::new(todofile);

        for (pos, line) in self.todo.iter().enumerate(){
            let line = if args[0] == (pos + 1).to_string(){
                let mut entry = Entry::read_line(line);
                entry.todo_entry = args[1].clone();
                entry.file_line()
            } else {
                format!("{}\n", line)
            };
            buffer 
                .write_all(line.as_bytes())
                .expect("unable to write data");
            }
    }
}

const TODO_HELP: &str = "USAGE: todo [COMMAND] [ARGUMENTS] Todo is a super fast and simple task organizer written in rust 
Available commands:
    -add [TASK/s]
    adds new task/s
    Exaple: todo add \"buy carrots\"
    -edit [INDEX] [EDITED TASK/s]
    edits an existing task/s
    example: todo edit 1 banana
    -list:
    list all tasks, ex: todo list 
    -done:
     mark task as done, ex. todo done 2 3
    -rm [INDEX]
        removes a task
        example: todo rm 4
    -reset : delete all tasks
    -restore: restore recent backup after reset 
    -sort: sorts completed & uncompleted tasks
    ex: todo sort 

    -raw [todo/done]
    prints nothing but done/incompleted tasks in plain text,
    ex: todo raw done 
    ";

pub fn help(){
    println!("{}", TODO_HELP);
    
}


