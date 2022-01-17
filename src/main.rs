extern crate ncurses;

use ncurses::*;
use std::fs::File;
use std::io::{ self, BufRead, ErrorKind, Write };

const COLOR_BACKGROUND: i16 = 15;
const COLOR_KEYWORD: i16 = 0; 
const COLOR_REGULAR_PAIR: i16 = 0;
const COLOR_PAIR_HIGHLIGHT: i16 = 2;

// constants::KEY_UP
// constants::KEY_DOWN
// constants::KEY_ENTER
// constants::KEY_BACKSPACE

#[derive(Default)]
struct Ui {
  height: i32,
  width: i32,
  key: Option<i32>
}

#[derive(Debug)]
enum TodoStatus {
  Todo,
  Done
}

impl Ui {
  fn layout(&mut self) {
    getmaxyx(stdscr(), &mut self.height, &mut self.width);
  }

  fn edit_field(&mut self, buffer: &mut String, cursor: &mut usize) {
    if *cursor > buffer.len() {
      *cursor = buffer.len()
    }

    if let Some(key) = self.key.take() {
      match key {
        32..=126 => {
          if *cursor >= buffer.len() 
          { buffer.push(key as u8 as char) } else 
          { buffer.insert(*cursor, key as u8 as char) }
          *cursor += 1
        }
        constants::KEY_LEFT => {
          if *cursor > 0 {
            *cursor += 1
          }
        }
        constants::KEY_RIGHT => {
          if *cursor < buffer.len() {
            *cursor += 1
          }
        }
        constants::KEY_BACKSPACE => {
          if *cursor > 0 {
            *cursor -= 1;
            if *cursor < buffer.len() {
              buffer.remove(*cursor);
            }
          }
        }
        constants::KEY_DC => {
          if *cursor < buffer.len() {
            buffer.remove(*cursor);
          }
        }
        _ => {
          self.key = Some(key)
        }
      }
    }

    {
      mv(self.height - 1, 0);
      attron(COLOR_PAIR(COLOR_REGULAR_PAIR) | A_BOLD());
      addstr(buffer);
      attroff(COLOR_PAIR(COLOR_REGULAR_PAIR) | A_BOLD());
    }

    {
      mv(self.height - 1, 0 + *cursor as i32);
      attron(COLOR_PAIR(COLOR_PAIR_HIGHLIGHT));
      addstr(buffer.get(*cursor..=*cursor).unwrap_or(" "));
      attroff(COLOR_PAIR(COLOR_PAIR_HIGHLIGHT));
    }
  }
}

impl TodoStatus {
  fn toggle(&self) -> Self {
    match self {
      Self::Todo => Self::Done,
      Self::Done => Self::Todo,
    }
  }
}

fn uplist(todos: &Vec<(TodoStatus, String)>, todo_curr: &mut usize) {
  if *todo_curr > 0 { *todo_curr -= 1 } 
  else { if todos.len() > 0 { *todo_curr = todos.len() - 1 } }
}

fn dwlist(todos: &Vec<(TodoStatus, String)>, todo_curr: &mut usize) {
  if todos.len() > 0 && *todo_curr < (todos.len() - 1) { *todo_curr += 1 }
  else { *todo_curr = 0 }
}

fn marktd(todos: &mut Vec<(TodoStatus, String)>, todo_curr: usize) {
  if todos.len() > todo_curr 
  { let (mark, content) = &todos[todo_curr];
    todos[todo_curr] = (TodoStatus::toggle(mark), String::from(content)) } 
}

fn delete(todos: &mut Vec<(TodoStatus, String)>, todo_curr: &mut usize) {
  if todos.len() > 0 { 
    todos.remove(*todo_curr); 
    
    if todos.len() == *todo_curr 
    && todos.len() != 0
    { *todo_curr -= 1 }
  } 
}

fn parse_line(line: &str) -> Option<(TodoStatus, &str)>  {
  let todo = line.strip_prefix("Todo,").map(|content| (TodoStatus::Todo, content));
  let done = line.strip_prefix("Done,").map(|content| (TodoStatus::Done, content));
  todo.or(done)
}

fn parse_to_string(status: &TodoStatus, content: &String) -> String {
  match status {
    TodoStatus::Todo => format!("Todo,{}", content),
    TodoStatus::Done => format!("Done,{}", content)
  }
}

fn load_todos(todos: &mut Vec<(TodoStatus, String)>, file_path: &str) -> io::Result<()> {
  let file = File::open(file_path)?;

  for (index, line) in io::BufReader::new(file).lines().enumerate() {
    match parse_line(&line?) {
      // i can abstract this?
      Some((TodoStatus::Todo, content)) => todos.push((TodoStatus::Todo, content.to_string())),
      Some((TodoStatus::Done, content)) => todos.push((TodoStatus::Done, content.to_string())),
      _ => {
        eprintln!("ERROR: at {}:{}", file_path, index + 1);
        std::process::exit(1);
      }
    }
  } Ok(())
}

fn save_todos(todos: &[(TodoStatus, String)], file_path: &str) {
  let mut file = File::create(file_path).unwrap();
  
  for (status, todo) in todos {
    writeln!(file, "{}", parse_to_string(status, todo)).unwrap()
  }
}

fn main() {
  initscr();
  noecho();
  timeout(16);
  // keypad(stdscr(), true);

  start_color();
  init_pair(COLOR_REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK);
  init_pair(COLOR_PAIR_HIGHLIGHT, COLOR_KEYWORD, COLOR_BACKGROUND);
  curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
  
  let mut quit = false;
  let mut ui = Ui::default();

  let mut todos = Vec::<(TodoStatus, String)>::new();
  let mut todo_curr: usize = 0;

  let mut editing = false;
  let mut editing_cursor = 0;

  match load_todos(&mut todos, "teste.txt") {
    Ok(()) => (),
    Err(err) => {
      if err.kind() == ErrorKind::NotFound {
        // temporary
        println!("new file created")
      } else {
        panic!(
          "{}", err
        )
      }
    }
  }

  while !quit {
    erase();
    ui.layout();

    attron(COLOR_PAIR(2));
    
    mvaddstr(
      ui.height - 2, 
      0, 
      &format!("todos: {} | current: {}", 
        todos.len(), 
        todo_curr+1)
    );

    attroff(COLOR_PAIR(2));
    
    for (index, (marked, content)) in todos.iter_mut().enumerate() {
      mv(index as i32, 0);
      
      let todo = &format!("[{}] {}", if matches!(marked, TodoStatus::Done) {"x"} else {" "}, content); 
       
      if index == todo_curr {
        attron(COLOR_PAIR(2) | A_BOLD());
        addstr(todo);
        attroff(COLOR_PAIR(2) | A_BOLD());
        
        if editing {
          ui.edit_field(content, &mut editing_cursor);
  
          if let Some('\n') = ui.key.map(|x| x as u8 as char) {
            editing = false
          }
        }
      } else {
        addstr(todo);
      }
      
    } 

    if let Some(key) = ui.key.take() {
      match key as u8 as char {
        'A' => uplist(&todos, &mut todo_curr),
        'B' => dwlist(&todos, &mut todo_curr),
        'D' => marktd(&mut todos, todo_curr),
        'C' => delete(&mut todos, &mut todo_curr),
        ';' => {
          todos.insert(0, (TodoStatus::Todo, String::new())); 
          editing_cursor = 0;
          editing = true;
        },
        _   => ui.key = Some(key)
      }
    }

    if let Some('q') = ui.key.map(|x| x as u8 as char) {
      quit = true
    }

    let key = getch();

    if key != ERR {
      ui.key = Some(key)
    }
  }

  save_todos(&todos, "teste.txt");
  endwin();
}