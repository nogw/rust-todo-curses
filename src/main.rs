extern crate ncurses;

use chrono;
use ncurses::*;
use std::fs::File;
use std::io::{self, BufRead, ErrorKind, Write};

const REGULAR_PAIR: i16 = 0;
const HIGHLIGHT_PAIR: i16 = 1;

// constants::KEY_UP
// constants::KEY_DOWN
// constants::KEY_ENTER
// constants::KEY_BACKSPACE

#[derive(Default)]
struct Ui {
  height: i32,
  width: i32,
  key: Option<i32>,
}

#[derive(Debug)]
enum TodoStatus {
  Todo,
  Done,
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
          if *cursor >= buffer.len() {
            buffer.push(key as u8 as char)
          } else {
            buffer.insert(*cursor, key as u8 as char)
          }
          *cursor += 1
        }

        constants::KEY_LEFT => {
          if *cursor > 0 {
            *cursor -= 1
          }
        }

        constants::KEY_RIGHT => {
          if *cursor < buffer.len() {
            *cursor += 1
          }
        }

        // it only works if you press CTRL at the same time
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

        _ => self.key = Some(key),
      }
    }

    {
      mv(self.height - 1, 0);
      attron(COLOR_PAIR(0) | A_BOLD());
      addstr(buffer);
      attroff(COLOR_PAIR(0) | A_BOLD());
    }

    {
      mv(self.height - 1, 0 + *cursor as i32);
      attron(COLOR_PAIR(1));
      addstr(buffer.get(*cursor..=*cursor).unwrap_or(" "));
      attroff(COLOR_PAIR(1));
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

fn uplist(todos: &Vec<(TodoStatus, String, String)>, todo_curr: &mut usize) {
  if *todo_curr > 0 {
    *todo_curr -= 1
  } else {
    if todos.len() > 0 {
      *todo_curr = todos.len() - 1
    }
  }
}

fn dwlist(todos: &Vec<(TodoStatus, String, String)>, todo_curr: &mut usize) {
  if todos.len() > 0 && *todo_curr < (todos.len() - 1) {
    *todo_curr += 1
  } else {
    *todo_curr = 0
  }
}

fn marktd(todos: &mut Vec<(TodoStatus, String, String)>, todo_curr: usize) {
  if todos.len() > todo_curr {
    let (mark, a, b) = &todos[todo_curr];
    todos[todo_curr] = (TodoStatus::toggle(mark), a.to_string(), b.to_string())
  }
}

fn delete(todos: &mut Vec<(TodoStatus, String, String)>, todo_curr: &mut usize) {
  if todos.len() > 0 {
    todos.remove(*todo_curr);

    if todos.len() == *todo_curr && todos.len() != 0 {
      *todo_curr -= 1
    }
  }
}

// looks bad
fn parse_line(line: &str) -> Option<(TodoStatus, String, String)> {
  let t = |b: TodoStatus, a: Vec<&str>| (b, a[0].to_string(), a[1].to_string());

  let todo = line
    .strip_prefix("Todo,")
    .map(|content| t(TodoStatus::Todo, content.split(",").collect()));
  let done = line
    .strip_prefix("Done,")
    .map(|content| t(TodoStatus::Done, content.split(",").collect()));
  todo.or(done)
}

fn parse_to_string(status: &TodoStatus, content: &String, time: &String) -> String {
  match status {
    TodoStatus::Todo => format!("Todo,{},{}", content, time),
    TodoStatus::Done => format!("Done,{},{}", content, time),
  }
}

fn load_todos(todos: &mut Vec<(TodoStatus, String, String)>, file_path: &str) -> io::Result<()> {
  let file = File::open(file_path)?;

  for (index, line) in io::BufReader::new(file).lines().enumerate() {
    match parse_line(&line?) {
      // i can abstract this?
      Some((TodoStatus::Todo, content, time)) => {
        todos.push((TodoStatus::Todo, content.to_string(), time.to_string()))
      }
      Some((TodoStatus::Done, content, time)) => {
        todos.push((TodoStatus::Done, content.to_string(), time.to_string()))
      }
      _ => {
        eprintln!("ERROR: at {}:{}", file_path, index + 1);
        std::process::exit(1);
      }
    }
  }
  Ok(())
}

fn save_todos(todos: &[(TodoStatus, String, String)], file_path: &str) {
  let mut file = File::create(file_path).unwrap();
  for (status, todo, time) in todos {
    if todo.len() > 0 && !matches!(status, TodoStatus::Done) {
      writeln!(file, "{}", parse_to_string(status, todo, time)).unwrap()
    }
  }
}

fn main() {
  initscr();
  noecho();
  keypad(stdscr(), true);
  timeout(16);
  curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

  start_color();
  init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK);
  init_pair(HIGHLIGHT_PAIR, COLOR_BLACK, COLOR_WHITE);

  let mut quit = false;
  let mut ui = Ui::default();

  let mut todos = Vec::<(TodoStatus, String, String)>::new();
  let mut todo_curr: usize = 0;

  let mut editing = false;
  let mut editing_cursor = 0;

  match load_todos(&mut todos, "todos.txt") {
    Ok(()) => (),
    Err(err) => {
      if err.kind() == ErrorKind::NotFound {
        // temporary
        println!("new file created")
      } else {
        panic!("{}", err)
      }
    }
  }

  while !quit {
    erase();
    ui.layout();

    let message: String;

    if let Some((_, _, at)) = &todos.get(todo_curr) {
      message = format!("[todos: {}] created at: {}", todos.len(), at)
    } else {
      message = format!("Press 'E' to create a new todo")
    }

    let bar = subwin(stdscr(), 1, ui.width, ui.height - 2, 0);
    wattron(bar, COLOR_PAIR(1) | A_BOLD());
    waddstr(bar, &message);
    wbkgd(bar, COLOR_PAIR(1) | A_BOLD());
    wattroff(bar, COLOR_PAIR(1) | A_BOLD());

    for (index, (marked, content, _)) in todos.iter_mut().enumerate() {
      mv(index as i32, 0);

      // TODO: looks bad
      let todo = &format!(
        "[{}] {}",
        if matches!(marked, TodoStatus::Done) {
          "x"
        } else {
          " "
        },
        content
      );

      if index == todo_curr {
        attron(COLOR_PAIR(1) | A_BOLD());
        addstr(todo);
        attroff(COLOR_PAIR(1) | A_BOLD());

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
        'j' => uplist(&todos, &mut todo_curr),
        'k' => dwlist(&todos, &mut todo_curr),
        'd' => marktd(&mut todos, todo_curr),
        'a' => delete(&mut todos, &mut todo_curr),
        'e' => {
          let time = chrono::offset::Local::now().format("%b %d %H:%M:%S");
          todos.insert(0, (TodoStatus::Todo, String::new(), time.to_string()));
          editing_cursor = 0;
          editing = true;
        }
        'q' => quit = true,
        _ => ui.key = Some(key),
      }
    }

    let key = getch();

    if key != ERR {
      ui.key = Some(key)
    }
  }

  save_todos(&todos, "todos.txt");
  endwin();
}
