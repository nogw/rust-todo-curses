extern crate ncurses;

use ncurses::*;

const COLOR_BACKGROUND: i16 = 20;
const COLOR_KEYWORD: i16 = 15; 
const COLOR_PAIR_KEYWORD: i16 = 2;

// constants::KEY_UP
// constants::KEY_DOWN
// constants::KEY_ENTER
// constants::KEY_BACKSPACE

#[derive(Default)]
struct Ui {
  key: Option<i32>
}

fn uplist(todos: &Vec<(bool, &str)>, todo_curr: &mut usize) {
  if *todo_curr > 0 { *todo_curr -= 1 } 
  else { if todos.len() > 0 { *todo_curr = todos.len() - 1 } }
}

fn dwlist(todos: &Vec<(bool, &str)>, todo_curr: &mut usize) {
  if todos.len() > 0 && *todo_curr < (todos.len() - 1) { *todo_curr += 1 }
  else { *todo_curr = 0 }
}

fn marktd(todos: &mut Vec<(bool, &str)>, todo_curr: usize) {
  if todos.len() > todo_curr 
  { let (mark, content) = todos[todo_curr];
    todos[todo_curr] = (!mark, content) } 
}

fn delete(todos: &mut Vec<(bool, &str)>, todo_curr: &mut usize) {
  if todos.len() > 0 { 
    todos.remove(*todo_curr); 
    
    if todos.len() == *todo_curr 
    && todos.len() != 0
    { *todo_curr -= 1 }
  } 
}

fn main() {
  initscr();
  noecho();
  timeout(16);

  start_color();
  init_pair(COLOR_PAIR_KEYWORD, COLOR_KEYWORD, COLOR_BACKGROUND);
  curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
  
  let mut quit = false;
  let mut ui = Ui::default();

  let mut todos = Vec::from(
    [(true, "Atodo"), 
     (false, "Btodo"), 
     (false, "Ctodo")]);

  let mut todo_curr: usize = 0;

  while !quit {
    erase();
    addstr(&format!("todos: {} | curr: {} | eq: {}", todos.len(), todo_curr, (if todos.len() == todo_curr + 1 { "true" } else {"false"})));

    for (index, (marked, content)) in todos.iter_mut().enumerate() {
      mv((index + 1) as i32, 0);
      
      let todo = &format!("[{}] {}", if *marked {"x"} else {" "}, content); 
  
      if index == todo_curr {
        attron(COLOR_PAIR(2));
        addstr(todo);
        attroff(COLOR_PAIR(2));
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

  endwin();
}