use std::str::FromStr;
use std::string::ToString;
use crate::tree::{Tree};
use crate::display::Display;
use crate::editor::{BACKGROUND_GREEN, FOREST_BORDERS};
use std::time::{Duration, Instant};
use std::io::{Read, Write};
use std::fs::{OpenOptions};
use rand::{Rng};
use termion::terminal_size;
use termion::async_stdin;
use termion::event::{Event, Key};

/// Error message when the screen is too small.
const GROW_SMALL_SCREEN_ERROR: &str = "The screen is too small, so the editor cannot be displayed properly. Make it larger (at least 25x26)";

/// Positive messages that are displayed each 5 minutes.
const POSITIVE: [&str; 3] = ["You're doing great, keep it up!", 
                             "You're getting closed, good job!",
                             "Why are you reading this? Get back to work!"];

/// The ammount of time used to grow a tree.
pub struct GrowthTime { // This feels kinda stupid, I should just use minutes
    pub h: u64,         // Also u64 feels really unnecessary, probably an u16 would be better
    pub m: u64,
}

impl GrowthTime {
    /// Returns the ammount of minutes equivalent to the GrowthTime
    pub fn to_min(&self) -> u64 {
        return self.h * 60 + self.m;
    }
}

impl FromStr for GrowthTime {
    type Err = String;

    /// Parse a time duration from a string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();

        if parts.len() != 2 {
            return Err("Failed to parse time: incorrect number of components".to_string());
        }

        let hh: u64 = match parts[0].parse() { 
        Ok(x) => { x }  
        Err(x) => { return Err(format!("Failed to parse time (hh): {}", x)); }
        };
        
        let mm: u64 = match parts[1].parse() { 
        Ok(x) => { x }  
        Err(x) => { return Err(format!("Failed to parse time (mm): {}", x)); }
        };

        Ok(GrowthTime {
            h: hh,
            m: mm,
        })
    }
}

impl ToString for GrowthTime {
    /// Convert a time duration to a string.
    fn to_string(&self) -> String {
        format!("{}:{}", self.h, self.m)
    }
}

/// Grow a tree. This implies waiting for the ammount of time requested by the user,
/// ocasionally send positive messages, and display a fancy menu if nogui is true.
pub fn grow_tree(chosen_tree: Tree, label: String, time: GrowthTime, nogui: bool) {
    if nogui {
        println!("Started growing your tree!");
        println!("If you ever want to cancel, you can CTRL+C");
        println!("But then your tree will die ;(");
    }

    let start = Instant::now();
    let target_duration = Duration::from_secs(time.h * 60 * 60 + time.m * 60);

    let mut last_positivity = target_duration.as_secs();
    let mut positive_message = String::new();

    let mut rng = rand::thread_rng();

    let mut gui = if nogui { None } else { Some((Display::new(), async_stdin().bytes())) };

    let mut exit_program = false;

    while start.elapsed() < target_duration && !exit_program {
        let remaining = (target_duration - start.elapsed()).as_secs();

        if remaining < last_positivity && remaining >= 3600 && remaining % 3600 == 0 {
            last_positivity = remaining;
            positive_message = format!("Hang in there! You got {}h left!", remaining / 3600);
            if nogui {
                println!("{}", positive_message);
            }
        } else if remaining < last_positivity && remaining < 3600 && remaining % (10 * 60) == 0 {
            last_positivity = remaining;
            positive_message = format!("You're close! You got {}m left!", remaining / 60);
            if nogui {
                println!("{}", positive_message);
            }
        } else if remaining < last_positivity && remaining % (5 * 60) == 0 {
            last_positivity = remaining;
            positive_message = POSITIVE[rng.gen::<usize>() % POSITIVE.len()].to_string();
            if nogui {
                println!("{}", positive_message);
            }
        }
        
        if let Some((ref mut gui, ref mut stdin)) = gui {
            let (width, height) = terminal_size().unwrap();
            let (width, height) = (width as usize, height as usize);

            gui.clear_screen(BACKGROUND_GREEN);

            let mut returned_none = false;
            while !returned_none {
                let key = stdin.next();
                match key {
                Some(k) => {
                    let e = termion::event::parse_event(k.unwrap(), stdin);
                    match e {
                    Ok(Event::Key(Key::Ctrl('c'))) => { exit_program = true; } 
                    _ => {}
                    }
                }
                None => { returned_none = true; }
                }
            }

            if width < 25 || height < 26 {
                gui.fit_string_to_box_hard_wrap(1, 1, width, height, BACKGROUND_GREEN, GROW_SMALL_SCREEN_ERROR);
            } else {
                let middle_col = (width + 1) / 2;
                
                for i in 1..height+1 {
                    gui.draw_pixel(i, 1, FOREST_BORDERS);
                    gui.draw_pixel(i, width, FOREST_BORDERS);
                    if i < height - 7 {
                        gui.draw_pixel(i, middle_col, FOREST_BORDERS);
                    }
                }
                
                for i in 1..width + 1 {
                    gui.draw_pixel(1, i, FOREST_BORDERS);
                    gui.draw_pixel(height, i, FOREST_BORDERS);
                    gui.draw_pixel(height - 7, i, FOREST_BORDERS);
                    gui.draw_pixel(9, i, FOREST_BORDERS);
                }

                for i in 0..7 {
                    gui.draw_pixel(6, middle_col - 3 + i, FOREST_BORDERS);
                    gui.draw_pixel(6 + i, middle_col - 3, FOREST_BORDERS);
                    gui.draw_pixel(12, middle_col - 3 + i, FOREST_BORDERS);
                    gui.draw_pixel(6 + i, middle_col + 3, FOREST_BORDERS);
                }
            
                for l in 0..5 {
                    for c in 0..5 {
                        gui.draw_pixel(7 + l, middle_col - 2 + c, chosen_tree.cells[l][c]);
                    }
                }
            
                gui.fit_string_to_box(height - 6, 2, width - 2, 6, BACKGROUND_GREEN, &positive_message);
                gui.draw_string(3, 3, BACKGROUND_GREEN, "left:");
                gui.draw_string(4, 3, BACKGROUND_GREEN, format!("{:02}:{:02}:{:02}", remaining / 3600, remaining / 60 % 60, remaining % 60).as_str());
            }

            gui.display();
        }
        
        std::thread::sleep(Duration::from_millis(50));
    }

    if !exit_program { // the user actually waited, so we must register this W
        let home = std::env::var("HOME");

        let home = match home {
        Ok(x) => { x }
        Err(x) => { println!("Failed to save data: {}", x); std::process::exit(1); }
        };
        
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(home + &"/.rusty-forest/stats.conf");

        let mut file_res = match file {
        Err(x) => { println!("Failed to open stats file: {}", x); std::process::exit(1); }
        Ok(x)  => { x }
        };
        
        file_res.write_all(format!("{}/{}/{}/{}\n", time.to_string(), label, chrono::offset::Local::now().timestamp(), chosen_tree.to_string()).as_bytes())
            .expect("Failed to write to file");
    }
} 

