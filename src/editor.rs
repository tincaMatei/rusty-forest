extern crate termion;

use crate::tree::Tree;
use termion::async_stdin;
use termion::event::{Event, Key};
use termion::terminal_size;
use std::io::{Read};
use std::thread;
use std::time::Duration;
use crate::display::Display;
use crate::tree::Cell;

/// Error displayed when the screen is too small.
const SMALL_SCREEN_ERROR: &str = "The screen is too small, so the editor cannot be displayed properly. Make it larger (at least 22x30)";

/// Editor instructions displayed on the right side.
const INSTRUCTIONS: &str = "Walk around with the arrow keys. Change colors with the menu below. To draw a character, just press the character to print. For a clear square, use Space. After finishing this, press Enter. To exit the editor without saving anything, use CTRL+c.";

/// Instructions that appear when naming the tree.
const NAME_TREE: &str = "Now you should give a name to your tree. It should only contain letters, digits, spaces and '-' or '_'";

/// An enum used to hold the editor state. This will be either EditTree, which means 
/// that the editor is used to actually create the tree, and NameTree, which means that 
/// here, a name should be given to the tree. Essentially, there are two menus.
enum EditorState {
    EditTree,
    NameTree,
}

/// The color used on the background, the fg color is used on the text.
pub const BACKGROUND_GREEN: Cell = Cell {
    bg: (44, 77, 52), 
    fg: (117, 199, 139),
    symbol: ' ',
};

/// The color used for the borders on the editor or growth menus.
pub const FOREST_BORDERS: Cell = Cell {
    bg: (14, 48, 23),
    fg: (0, 0, 0),
    symbol: ' ',
};

/// Start the tree editor that returns the created tree.
pub fn run_tree_editor() -> Tree {
    let mut stdin = async_stdin().bytes();
    let mut exit_program = false;

    let mut display = Display::new();
    display.clear_screen(Cell::default());
    
    let mut state = EditorState::EditTree;
    let mut final_tree = Tree::default();

    let mut l_tree = 0;
    let mut c_tree = 0;

    let mut brush: Cell = Cell::default();

    let mut str_cursor: usize = 0;

    let mut banner: String = NAME_TREE.to_string();

    while !exit_program {
        let (width, height) = terminal_size().unwrap();
        let (width, height) = (width as usize, height as usize);

        let mut returned_none = false;
        
        while !returned_none {
            let key = stdin.next();
            match key {
            Some(k) => {
                let e = termion::event::parse_event(k.unwrap(), &mut stdin);
                match e {
                Ok(Event::Key(Key::Ctrl('c'))) => { exit_program = true; } 
                Ok(Event::Key(Key::Char('\n'))) => {
                    match state {
                    EditorState::EditTree => { state = EditorState::NameTree; }
                    EditorState::NameTree => {
                        if final_tree.name.is_empty() {
                            banner = "Please name your tree!".to_string();
                        } else {
                            exit_program = true;
                        }
                    }
                    }
               }

                Ok(Event::Key(Key::Up)) => {
                    match state {
                    EditorState::EditTree => {
                        if l_tree == 0 {
                            l_tree = 10;
                        } else {
                            l_tree = l_tree - 1;
                        }
                    }
                    _ => {}
                    }
                }
                
                Ok(Event::Key(Key::Down)) => {
                    match state {
                    EditorState::EditTree => {
                        l_tree = (l_tree + 1) % 11;
                    }
                    _ => {}
                    }
                }

                Ok(Event::Key(Key::Left)) => {
                    match state {
                    EditorState::EditTree => {
                        match l_tree {
                        0 | 1 | 2 | 3 | 4 => {
                            if c_tree == 0 {
                                c_tree = 4;
                            } else {
                                c_tree = c_tree - 1;
                            }
                        }
                        5  => { if brush.bg.0 > 0 { brush.bg.0 = brush.bg.0 - 1; } }
                        6  => { if brush.bg.1 > 0 { brush.bg.1 = brush.bg.1 - 1; } }
                        7  => { if brush.bg.2 > 0 { brush.bg.2 = brush.bg.2 - 1; } }
                        8  => { if brush.fg.0 > 0 { brush.fg.0 = brush.fg.0 - 1; } }
                        9  => { if brush.fg.1 > 0 { brush.fg.1 = brush.fg.1 - 1; } }
                        10 => { if brush.fg.2 > 0 { brush.fg.2 = brush.fg.2 - 1; } }
                        _ => {}
                        }
                    }
                    EditorState::NameTree => {
                        if str_cursor > 0 {
                            str_cursor = str_cursor - 1;
                        }
                    }
                    }
                }

                Ok(Event::Key(Key::Right)) => {
                    match state {
                    EditorState::EditTree => {
                        match l_tree {
                        0 | 1 | 2 | 3 | 4 => { c_tree = (c_tree + 1) % 5; }
                        5  => { if brush.bg.0 < 255 { brush.bg.0 = brush.bg.0 + 1; } }
                        6  => { if brush.bg.1 < 255 { brush.bg.1 = brush.bg.1 + 1; } }
                        7  => { if brush.bg.2 < 255 { brush.bg.2 = brush.bg.2 + 1; } }
                        8  => { if brush.fg.0 < 255 { brush.fg.0 = brush.fg.0 + 1; } }
                        9  => { if brush.fg.1 < 255 { brush.fg.1 = brush.fg.1 + 1; } }
                        10 => { if brush.fg.2 < 255 { brush.fg.2 = brush.fg.2 + 1; } }
                        _ => {}
                        }
                    }
                    EditorState::NameTree => {
                        if str_cursor < final_tree.name.len() {
                            str_cursor = str_cursor + 1;
                        }
                    }
                    }
                    
                }

                Ok(Event::Key(Key::Char(x))) => {
                    match state {
                    EditorState::EditTree => {
                        if l_tree < 5 {
                            brush.symbol = x;
                            final_tree.cells[l_tree][c_tree] = brush;
                        }
                    }
                    EditorState::NameTree => {
                        match x {
                        'a'..='z' | 'A'..='Z' | '0'..='9' | ' ' | '-' | '_' => {
                            final_tree.name.insert(str_cursor, x);
                            str_cursor = str_cursor + 1;
                        }
                        _ => {
                        }
                        }
                    }
                    }
                }
                
                Ok(Event::Key(Key::Backspace)) => {
                    match state {
                    EditorState::NameTree => {
                        if str_cursor > 0 {
                            str_cursor = str_cursor - 1;
                            final_tree.name.remove(str_cursor);
                        }
                    }
                    _ => {}
                    }
                }

                Ok(Event::Key(Key::Delete)) => {
                    match state {
                    EditorState::NameTree => {
                        if str_cursor < final_tree.name.len() {
                            final_tree.name.remove(str_cursor);
                        }
                    }
                    _ => {}
                    }
                }

                _ => {}
                }
            }
            None => { returned_none = true; }
            }
        }
        
        display.clear_screen(BACKGROUND_GREEN);
        if height < 22 || width < 30 { // The editor cannot be displayed properly
            let mut l: usize = 1;
            let mut c: usize = 1;
            

            for chr in SMALL_SCREEN_ERROR.bytes() {
                if l <= height && c <= width {
                    display.draw_pixel(l, c, BACKGROUND_GREEN.change_symbol(chr as char) );
                    c = c + 1;
                    if c > width {
                        c = 1;
                        l = l + 1;
                    }
                }
            }
        } else {
            match state {
            EditorState::EditTree => {
                for i in 1..width+1 {
                    display.draw_pixel(1, i, FOREST_BORDERS);
                    display.draw_pixel(height, i, FOREST_BORDERS);
                }
                
                for i in 1..height+1 {
                    display.draw_pixel(i, 8, FOREST_BORDERS);
                    display.draw_pixel(i, 9, FOREST_BORDERS);
                }
            
                for i in 1..7+1 {
                    display.draw_pixel(2, i, FOREST_BORDERS);
                    display.draw_pixel(1 + i, 1, FOREST_BORDERS);
                    display.draw_pixel(8, i, FOREST_BORDERS);
                    display.draw_pixel(1 + i, 7, FOREST_BORDERS);
                }
            
                for l in 0..5 {
                    for c in 0..5 {
                        display.draw_pixel(3 + l, 2 + c, final_tree.cells[l][c]);
                    }
                }

                display.draw_string( 9, 1, BACKGROUND_GREEN, "BG");
                display.draw_string(10, 1, BACKGROUND_GREEN, "Red");
                display.draw_string(11, 1, BACKGROUND_GREEN, "Green");
                display.draw_string(12, 1, BACKGROUND_GREEN, "Blue");
                display.draw_string(15, 1, BACKGROUND_GREEN, "FG");
                display.draw_string(16, 1, BACKGROUND_GREEN, "Red");
                display.draw_string(17, 1, BACKGROUND_GREEN, "Green");
                display.draw_string(18, 1, BACKGROUND_GREEN, "Blue");
                
                if l_tree < 5 {
                    let mut cursor_brush = brush;
                    cursor_brush.fg = (255, 255, 255);
                    cursor_brush.symbol = '*';
                    display.draw_pixel(l_tree + 3, c_tree + 2, cursor_brush);
                } else if 5 <= l_tree && l_tree <= 7 {
                    display.draw_string(5 + l_tree, 6, BACKGROUND_GREEN, "<>");
                } else {
                    display.draw_string(8 + l_tree, 6, BACKGROUND_GREEN, "<>");
                }
                
                let cost = final_tree.cost();
                let extended_instr = INSTRUCTIONS.to_owned() + &format!("\n\nTree cost: {:02}:{:02}", cost / 60, cost % 60);

                display.fit_string_to_box(2, 10, width - 9, height - 2, 
                    BACKGROUND_GREEN, &extended_instr);
                
                brush.symbol = ' ';
                display.draw_pixel(height - 1, 10, brush);
                display.draw_pixel(height - 1, 11, brush);
                
                display.draw_string(height - 1, 12, BACKGROUND_GREEN,
                    &format!("BG: {:?}", brush.bg));
                
                display.draw_pixel(height - 2, 10, Cell::bg(brush.fg.0, brush.fg.1, brush.fg.2));
                display.draw_pixel(height - 2, 11, Cell::bg(brush.fg.0, brush.fg.1, brush.fg.2));

                display.draw_string(height - 2, 12, BACKGROUND_GREEN,
                    &format!("FG: {:?}", brush.fg));
            }
            EditorState::NameTree => {
                for i in 1..width+1 {
                    display.draw_pixel(1, i, FOREST_BORDERS);
                    display.draw_pixel(6, i, FOREST_BORDERS);
                    display.draw_pixel(height, i, FOREST_BORDERS);
                }
                for i in 1..height+1 {
                    display.draw_pixel(i, 1, FOREST_BORDERS);
                    display.draw_pixel(i, width, FOREST_BORDERS);
                }
                display.fit_string_to_box(2, 2, width - 2, 4, BACKGROUND_GREEN, &banner);
                display.fit_string_to_box_hard_wrap(7, 2, width - 2, height - 6, BACKGROUND_GREEN, &final_tree.name);
            
                let curs_lin = str_cursor / (width - 2) + 7;
                let curs_col = str_cursor % (width - 2) + 2;
                display.draw_pixel(curs_lin, curs_col, Cell::bg(255, 255, 255));
            }
            }
        }

        display.display();
        thread::sleep(Duration::from_millis(50));
    }


    return final_tree;
}

