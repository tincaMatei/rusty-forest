use termion::raw::IntoRawMode;
use termion::{cursor};
use termion::screen::*;
use termion::terminal_size;
use termion::raw::RawTerminal;
use crate::tree::Cell;
use std::io::{Write, stdout, Stdout};

/// A struct to work with the display of the screen. At the creation, it will
/// automatically hide the cursor and make an alternate screen. They will be restored 
/// when dropped.
pub struct Display {
    /// width of the screen; it will be changed when displaying
    width: usize,
    /// height of the screen; it will be changed when displaying
    height: usize,
    /// the old buffer; used to display only the changed characters on the screen
    old_matrix: Vec<Vec<Cell> >,
    /// the buffer; when writing stuff on the screen, this will be modified
    matrix: Vec<Vec<Cell> >,
    /// a field containing an instance of stdout, wrapped in an AlternateScreen
    stdout: AlternateScreen<RawTerminal<Stdout> >,
    /// a HideCursor instance that will handle the cursor
    #[allow(dead_code)]
    hide_cursor: cursor::HideCursor<Stdout>,
}

impl Display {
    /// Create a new display. When doing this, the alternate screen is automatically 
    /// activated, and the cursor is hidden.
    pub fn new() -> Self {
        let (width, height) = terminal_size().unwrap();
        let old_matrix = vec![vec![Cell::default(); width as usize]; height as usize];
        let matrix = vec![vec![Cell::default(); width as usize]; height as usize];
        let mut screen = AlternateScreen::from(stdout().into_raw_mode().unwrap());
        
        write!(screen, "{}{}{}", termion::color::Fg(termion::color::Rgb(0, 0, 0)),
                               termion::color::Bg(termion::color::Rgb(0, 0, 0)),
                               termion::clear::All).expect("Failed to write");

        screen.flush().expect("Failed to flush");

        Display {
            width: width as usize,
            height: height as usize,
            old_matrix,
            matrix,
            stdout: screen,
            hide_cursor: cursor::HideCursor::from(stdout()),
        }
    }

    /// Draw a pixel on the buffer on the l'th line and c'th column.
    pub fn draw_pixel(&mut self, l: usize, c: usize, cell: Cell) {
        if 1 <= l && l <= self.height && 1 <= c && c <= self.width {
            self.matrix[l - 1][c - 1] = cell;
        }
    }

    /// Clear the entire screen with the background color of the cell.
    pub fn clear_screen(&mut self, cell: Cell) {
        for l in 0..self.height {
            for c in 0..self.width {
                self.matrix[l][c] = cell;
            }
        }
    }
    
    /// Draw a string at the given position (the l'th line, c'th column) using 
    /// the style in the cell.
    pub fn draw_string(&mut self, l: usize, mut c: usize, mut cell: Cell, to_write: &str) {
        for chr in to_write.chars() {
            if 1 <= l && l <= self.height && 1 <= c && c <= self.width {
                cell.symbol = chr;
                self.matrix[l - 1][c - 1] = cell;
            }
            c = c + 1;
        }
    }

    /// Fit the string inside a rectangle and try to handle the text wrapping.
    /// The string will be fitted in a box of the given width and height, with the 
    /// upper-left corner on the l'th line and c'th column. The style used will be the 
    /// one contained in cell.
    pub fn fit_string_to_box(&mut self, l: usize, c: usize, width: usize, height: usize, cell: Cell, to_write: &str) {
        let phrases = to_write.split('\n');
        
        let mut line: usize = 0;
        let mut col: usize = 0;

        for phrase in phrases {
            let words = phrase.split(' ');

            for mut word in words {
                while word.len() > 0 {
                    if col + word.len() <= width { // The word fits on the entire line
                        if line < height {
                            self.draw_string(l + line, c + col, cell, word);
                        }
                        col = col + word.len();
                        word = &word[0..0];
                    } else if word.len() <= width { // we can fit the entire word into the next line
                        line = line + 1;
                        col = 0;
                        if line < height {
                            self.draw_string(l + line, c + col, cell, word);
                        }
                        col = col + word.len();
                        word = &word[0..0];
                    } else { // here we should try to fit this as much as possible
                        let fitting = width - col;
                        if fitting <= word.len() {
                            if line < height {
                                self.draw_string(l + line, c + col, cell, &word[0..fitting]);
                            }
                            col = 0;
                            line = line + 1;
                            word = &word[fitting..];
                        } else {
                            if line < height {
                                self.draw_string(l + line, c + col, cell, word);
                            }
                            col = col + word.len();
                            word = &word[0..0];
                        }
                    }
                }
                
                // Here we should add a space
                col = col + 1;
                if col == width {
                    col = 0;
                    line = line + 1;
                }
            }
            
            // we should go to the next line afther writing a phrase
            col = 0;
            line = line + 1;
        }
    }

    /// Exactly like fit_string_to_box(), but cuts the words if they reach the right border.
    pub fn fit_string_to_box_hard_wrap(&mut self, l: usize, c: usize, width: usize, height: usize, mut cell: Cell, to_write: &str) {
        let mut line: usize = 0;
        let mut col: usize = 0;

        for chr in to_write.chars() {
            if line < height {
                cell.symbol = chr;
                self.draw_pixel(l + line, c + col, cell);
            }
            col = col + 1;
            if col == width {
                line = line + 1;
                col = 0;
            }
        }
    }

    /// Display all the modifications on the screen and handle the terminal size changes.
    pub fn display(&mut self) {
        let (width, height) = terminal_size().unwrap();
        let (width, height) = (width as usize, height as usize);
        
        for l in 1..height+1 {
            for c in 1..width+1 {
                if (self.width != width || self.height != self.height)
                && l <= self.height && c <= self.width {
                    let (r, g, b) = self.matrix[l - 1][c - 1].fg;
                    let (r2, g2, b2) = self.matrix[l - 1][c - 1].bg;
                    write!(self.stdout, "{}", termion::cursor::Goto(c as u16, l as u16))
                        .expect("Failed to write");
                    write!(self.stdout, "{}{}{}", termion::color::Fg(termion::color::Rgb(r, g, b)),
                                                  termion::color::Bg(termion::color::Rgb(r2, g2, b2)),
                                                  self.matrix[l - 1][c - 1].symbol)
                        .expect("Failed to write");
                } else if self.width != width || self.height != height {
                    write!(self.stdout, "{}", termion::cursor::Goto(c as u16, l as u16))
                        .expect("Failed to write");
                    write!(self.stdout, "{}{} ", termion::color::Fg(termion::color::Rgb(0, 0, 0)),
                                                 termion::color::Bg(termion::color::Rgb(0, 0, 0)))
                        .expect("Failed to write");
                } else if l <= self.height && c <= self.width &&
                   self.old_matrix[l - 1][c - 1] != self.matrix[l - 1][c - 1]{
                    let (r, g, b) = self.matrix[l - 1][c - 1].fg;
                    let (r2, g2, b2) = self.matrix[l - 1][c - 1].bg;
                    write!(self.stdout, "{}", termion::cursor::Goto(c as u16, l as u16))
                        .expect("Failed to write");
                    write!(self.stdout, "{}{}{}", termion::color::Fg(termion::color::Rgb(r, g, b)),
                                                  termion::color::Bg(termion::color::Rgb(r2, g2, b2)),
                                                  self.matrix[l - 1][c - 1].symbol)
                        .expect("Failed to write");
                } else if !(l <= self.height && c <= self.width) {
                    write!(self.stdout, "{}", termion::cursor::Goto(c as u16, l as u16))
                        .expect("Failed to write");
                    write!(self.stdout, "{}{} ", termion::color::Fg(termion::color::Rgb(0, 0, 0)),
                                                 termion::color::Bg(termion::color::Rgb(0, 0, 0)))
                        .expect("Failed to write");
                }
            }
        }
        
        self.width = width;
        self.height = height;

        self.matrix.resize(self.height, Vec::new());
        for line in 0..height {
            self.matrix[line].resize(self.width, Cell::default());
        }
        
        self.old_matrix = self.matrix.clone();
        
        self.stdout.flush()
            .expect("Failed to flush");
    }
    
    /// Used for debug purposes.
    pub fn screen_shot(&self) {
        eprintln!("DEBUG:\n");
        for i in 0..self.height {
            eprintln!("{:?}", self.matrix[i][0]);
        }
    }
}

