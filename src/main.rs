extern crate ncurses;

use ncurses::*;

const COLOR_BACKGROUND: i16 = 20;
const COLOR_KEYWORD: i16 = 15; 
const COLOR_PAIR_KEYWORD: i16 = 2;

fn uplist(todos: &Vec<(bool, &str)>, todo_curr: &mut usize) {
  if *todo_curr > 0 { *todo_curr -= 1 } 
  else { *todo_curr = todos.len() - 1 }
}

fn dwlist(todos: &Vec<(bool, &str)>, todo_curr: &mut usize) {
  if *todo_curr < todos.len() - 1 { *todo_curr += 1 }
  else { *todo_curr = 0 }
}

fn main() {
  initscr();
  noecho();
  timeout(16);

  start_color();
  init_pair(COLOR_PAIR_KEYWORD, COLOR_KEYWORD, COLOR_BACKGROUND);
  curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
  
  let mut quit = false;
  
  let mut todos = Vec::from(
    [(true, "Atodo"), 
     (false, "Btodo"), 
     (false, "Ctodo")]);
  let mut todo_curr = 0;

  while !quit {
    erase();
    
    for (index, (marked, content)) in todos.iter_mut().enumerate() {
      mv(index as i32, 0);
      
      let todo = &format!("[{}] {}", if *marked {"x"} else {" "}, content); 
  
      if index == todo_curr {
        attron(COLOR_PAIR(2));
        addstr(todo);
        attroff(COLOR_PAIR(2));
      } else {
        addstr(todo);
      }
    } 

    let key = getch() as u8 as char;

    match key {
      'A' => uplist(&todos, &mut todo_curr),
      'B' => dwlist(&todos, &mut todo_curr),
      'q' => quit = true,
      _   => {}
    }
  }

  endwin();
}