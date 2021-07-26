use termion::raw::IntoRawMode;
use termion::async_stdin;
use termion::{color, style, cursor};
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::screen::*;
use termion::terminal_size;
use termion::raw::RawTerminal;
use crate::tree::Cell;
use std::io::{Read, Write, stdout, Stdout};

pub struct Display {
    width: usize,
    height: usize,
    old_matrix: Vec<Vec<Cell> >,
    matrix: Vec<Vec<Cell> >,
    stdout: AlternateScreen<RawTerminal<Stdout> >,
    hide_cursor: cursor::HideCursor<Stdout>,
}

impl Display {
    pub fn new() -> Self {
        let (width, height) = terminal_size().unwrap();
        let mut old_matrix = vec![vec![Cell::default(); width as usize]; height as usize];
        let mut matrix = vec![vec![Cell::default(); width as usize]; height as usize];
        let mut screen = AlternateScreen::from(stdout().into_raw_mode().unwrap());
        
        write!(screen, "{}{}{}", termion::color::Fg(termion::color::Rgb(0, 0, 0)),
                               termion::color::Bg(termion::color::Rgb(0, 0, 0)),
                               termion::clear::All);

        screen.flush();

        Display {
            width: width as usize,
            height: height as usize,
            old_matrix,
            matrix,
            stdout: screen,
            hide_cursor: cursor::HideCursor::from(stdout()),
        }
    }

    pub fn draw_pixel(&mut self, l: usize, c: usize, cell: Cell) {
        if 1 <= l && l <= self.height && 1 <= c && c <= self.width {
            self.matrix[l - 1][c - 1] = cell;
        }
    }

    pub fn clear_screen(&mut self, cell: Cell) {
        for l in 0..self.height {
            for c in 0..self.width {
                self.matrix[l][c] = cell;
            }
        }
    }

    pub fn draw_string(&mut self, mut l: usize, mut c: usize, mut cell: Cell, to_write: &str) {
        for chr in to_write.chars() {
            if 1 <= l && l <= self.height && 1 <= c && c <= self.width {
                cell.symbol = chr;
                self.matrix[l - 1][c - 1] = cell;
            }
            c = c + 1;
        }
    }

    pub fn fit_string_to_box(&mut self, l: usize, c: usize, width: usize, height: usize, cell: Cell, to_write: &str) {
        let phrases = to_write.split('\n');
        
        let mut line: usize = 0;
        let mut col: usize = 0;

        for phrase in phrases {
            let mut words = phrase.split(' ');

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

    pub fn display(&mut self) {
        let (width, height) = terminal_size().unwrap();
        let (width, height) = (width as usize, height as usize);
        
        for l in 1..height+1 {
            for c in 1..width+1 {
                if (self.width != width || self.height != self.height)
                && l <= self.height && c <= self.width {
                    let (r, g, b) = self.matrix[l - 1][c - 1].fg;
                    let (r2, g2, b2) = self.matrix[l - 1][c - 1].bg;
                    write!(self.stdout, "{}", termion::cursor::Goto(c as u16, l as u16));
                    write!(self.stdout, "{}{}{}", termion::color::Fg(termion::color::Rgb(r, g, b)),
                                                  termion::color::Bg(termion::color::Rgb(r2, g2, b2)),
                                                  self.matrix[l - 1][c - 1].symbol);
                } else if self.width != width || self.height != height {
                    write!(self.stdout, "{}", termion::cursor::Goto(c as u16, l as u16));
                    write!(self.stdout, "{}{} ", termion::color::Fg(termion::color::Rgb(0, 0, 0)),
                                                 termion::color::Bg(termion::color::Rgb(0, 0, 0)));
                } else if l <= self.height && c <= self.width &&
                   self.old_matrix[l - 1][c - 1] != self.matrix[l - 1][c - 1]{
                    let (r, g, b) = self.matrix[l - 1][c - 1].fg;
                    let (r2, g2, b2) = self.matrix[l - 1][c - 1].bg;
                    write!(self.stdout, "{}", termion::cursor::Goto(c as u16, l as u16));
                    write!(self.stdout, "{}{}{}", termion::color::Fg(termion::color::Rgb(r, g, b)),
                                                  termion::color::Bg(termion::color::Rgb(r2, g2, b2)),
                                                  self.matrix[l - 1][c - 1].symbol);
                } else if !(l <= self.height && c <= self.width) {
                    write!(self.stdout, "{}", termion::cursor::Goto(c as u16, l as u16));
                    write!(self.stdout, "{}{} ", termion::color::Fg(termion::color::Rgb(0, 0, 0)),
                                                 termion::color::Bg(termion::color::Rgb(0, 0, 0)));
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
        
        self.stdout.flush();
    }

    pub fn screen_shot(&self) {
        eprintln!("DEBUG:\n");
        for i in 0..self.height {
            eprintln!("{:?}", self.matrix[i][0]);
        }
    }
}

