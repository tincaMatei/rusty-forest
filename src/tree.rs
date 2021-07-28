use termion::{color};
use std::str::FromStr;
use std::io::{Write, stdout};
use std::string::ToString;
use std::fs::{self, File, DirBuilder};
use regex::Regex;
use std::default::Default;
use std::cmp;
use crate::grow::GrowthTime;

/// Handle all things about trees, which could be useful also for the GUI,
/// or for the time management.
///
/// The trees and grown trees have their own format used to be stored in files.
///
/// The tree itself has the format `<hex-string>:<name>`, where the `<hex-string>` has 
/// data about the appearance of the tree. Each group of 14 characters is specific to a 
/// cell in the tree. Each group of two in the string is a byte, so there are 7 bytes for 
/// each cell. In order, the 7 bytes of a cell represent: background red, background green, 
/// background blue, foreground red, foreground green, foreground blue and the symbol.
/// The cells in order are taken from the first line, from the top to the bottom, and 
/// for each line, they're taken in order from left to right.
///
/// The name should consist only of alphabet letters, digits, empty space, hyphens or underlines.

/// A cell containing the RGB-value of the background, foreground, and the character.
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Cell {
    pub bg: (u8, u8, u8),
    pub fg: (u8, u8, u8),
    pub symbol: char,
}

impl Cell {
    /// Create a new cell by listing all the fields necessary.
    pub fn new(bgr: u8, bgg: u8, bgb: u8, fgr: u8, fgg: u8, fgb: u8, symbol: char) -> Cell {
        Cell {
            bg: (bgr, bgg, bgb),
            fg: (fgr, fgg, fgb),
            symbol,
        }
    }

    /// Create a cell only with the background values.
    pub fn bg(r: u8, g: u8, b: u8) -> Cell {
        Cell {
            bg: (r, g, b),
            fg: (0, 0, 0),
            symbol: ' ',
        }
    }
    
    /// Create a new cell from another one by changing only the symbol.
    pub fn change_symbol(&self, chr: char) -> Cell {
        Cell {
            bg: self.bg,
            fg: self.fg,
            symbol: chr,
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            bg: (0, 0, 0),
            fg: (0, 0, 0),
            symbol: ' ',
        }
    }
}

/// A tree, which is characterized by a 5x5 Cell matrix, which is the appearance of the tree, 
/// and its name.
#[derive(Debug, Clone)]
pub struct Tree {
    pub cells: Vec<Vec<Cell>>,
    pub name: String,
}

impl Default for Tree {
    fn default() -> Self {
        let cells = vec![vec![Cell::default(); 5]; 5];
        Tree {
            cells,
            name: String::new(),
        }
    }
}

impl Tree {
    /// Get the background color of the cell from the l'th line and c'th column 
    /// of the tree.
    pub fn get_bg_color(&self, l: usize, c: usize) -> color::Rgb {
        color::Rgb(self.cells[l][c].bg.0, self.cells[l][c].bg.1, self.cells[l][c].bg.2)
    }

    /// Get the foreground color of the cell from the l'th line and c'th column 
    /// of the tree.
    pub fn get_fg_color(&self, l: usize, c: usize) -> color::Rgb {
        color::Rgb(self.cells[l][c].fg.0, self.cells[l][c].fg.1, self.cells[l][c].fg.2)
    }

    /// Display the cell from the l'th line and c'th column at the position of the cursor.
    pub fn display_symbol(&self, l: usize, c: usize) {
        write!(stdout(), "{}{}{}", color::Bg(self.get_bg_color(l, c)), 
                                   color::Fg(self.get_fg_color(l, c)), 
                                   self.cells[l][c].symbol).expect("Failed to write");
    }

    /// Create a new tree from a hex string and name.
    fn new(bytes: Vec<u8>, name: String) -> Result<Tree, String> {
        if bytes.len() != 25 * 7 {
            return Err("Wrong number of bytes".to_string());
        }

        let mut arr: Vec<Vec<Cell>> = Vec::new();
        let mut last_byte = 0;

        for _ in 0..5 {
            let mut line: Vec<Cell> = Vec::new();
            for _ in 0..5 {
                line.push(Cell {
                    bg: (bytes[last_byte], bytes[last_byte + 1], bytes[last_byte + 2]),
                    fg: (bytes[last_byte + 3], bytes[last_byte + 4], bytes[last_byte + 5]),
                    symbol: bytes[last_byte + 6] as char,
                });
                last_byte += 7;
            }

            arr.push(line);
        }

        Ok(Tree {
            cells: arr,
            name
        })
    }
    
    /// Import a tree from a string that respects the tree format.
    pub fn import_tree(tree: String) -> Result<Tree, String> {
        if !Tree::is_legit(&tree) {
            return Err("The tree does not respect the format".to_string());
        }

        let parts: Vec<&str> = tree.split(":").collect();

        if parts.len() != 2 {
            return Err("Wrong number of ':'".to_string());
        }

        let tree_data = match hex::decode(parts[0]) {
        Ok(x) => { x }
        Err(x) => { return Err(format!("{}", x)); }
        };

        let tree_name = parts[1];

        Tree::new(tree_data, tree_name.to_string())
    }

    /// Returns true if the given tree string is correct.
    pub fn is_legit(tree: &String) -> bool {
        Regex::new("^[A-Fa-f0-9]{350}:[-_ a-zA-Z0-9]+$").unwrap().is_match(tree)
    }
    
    /// Calculates the time cost of a tree. In particular, a "greener" tree will be cheaper, 
    /// while a more colored tree will be more expensive.
    pub fn cost(&self) -> u64 {
        let base_cost = 15;
        let (mut sum_red_bg, mut sum_blue_bg): (u64, u64) = (0, 0);
        let (mut sum_red_fg, mut sum_blue_fg): (u64, u64) = (0, 0);
        for l in 0..5 {
            for c in 0..5 {
                sum_red_bg = sum_red_bg + (self.cells[l][c].bg.0 as u64);
                sum_red_fg = sum_red_fg + (self.cells[l][c].fg.0 as u64);
            
                sum_blue_bg = sum_blue_bg + (self.cells[l][c].bg.2 as u64);
                sum_blue_fg = sum_blue_fg + (self.cells[l][c].fg.2 as u64);
            }
        }
        
        let bg_cost = ((cmp::max(sum_red_bg, sum_blue_bg) as f64 / (255.0 * 5.0 * 5.0) * 12.0).floor() as u64) * 5;
        let fg_cost = ((cmp::max(sum_red_fg, sum_blue_fg) as f64 / (255.0 * 5.0 * 5.0) *  8.0).floor() as u64) * 5;

        base_cost + bg_cost + fg_cost
    }
}

impl ToString for Tree {
    /// Convert a tree to a string that respects the tree format
    fn to_string(&self) -> String {
        let mut res: Vec<u8> = Vec::new();
        for l in 0..5 {
            for c in 0..5 {
                res.push(self.cells[l][c].bg.0);
                res.push(self.cells[l][c].bg.1);
                res.push(self.cells[l][c].bg.2);
                res.push(self.cells[l][c].fg.0);
                res.push(self.cells[l][c].fg.1);
                res.push(self.cells[l][c].fg.2);
                res.push(self.cells[l][c].symbol as u8);
            }
        }

        hex::encode(res) + &":" + &self.name
    }
}

pub struct TreeCollection {
    pub collection: Vec<Tree>
}

impl TreeCollection {
    /// Load all trees from `~/.rusty-forest/trees.conf`.
    pub fn load() -> Self {
        check_directories().expect("Failed to check directories");
        
        let home = std::env::var("HOME");

        let home = match home {
        Ok(x) => { x }
        Err(_) => { return TreeCollection { collection: Vec::new() }; }
        };
        
        let mut trees: Vec<Tree> = Vec::new();
        
        trees.push(Tree::import_tree("0000000000002000000000000020000000000000200000000000002000000000000020000000000000201e6e00000000201e6e00000000201e6e0000000020000000000000201e6e00000000201e6e00000000201e6e00000000201e6e00000000201e6e00000000200000000000002000000000000020321e000000002000000000000020000000000000200000000000002000000000000020321e00000000200000000000002000000000000020:default-1".to_string()).unwrap());
        trees.push(Tree::import_tree("00000000000020000000000000201e6e00000000200000000000002000000000000020000000000000201e6e00000000201e6e00000000201e6e000000002000000000000020000000000000201e6e00000000201e6e00000000201e6e0000000020000000000000201e6e00000000201e6e00000000201e6e00000000201e6e00000000201e6e00000000200000000000002000000000000020321e00000000200000000000002000000000000020:default-2".to_string()).unwrap());
        trees.push(Tree::import_tree("00000000000020000000000000201e6e00000000200000000000002000000000000020000000000000201e6e00000000201e6e00000000201e6e00ff00006f000000000000201e6e00ff00006f1e6e00ff00006f1e6e00000000201e6e00000000201e6e00ff00006f0000000000002000000000000020321e000000002000000000000020000000000000200000000000002000000000000020321e00000000200000000000002000000000000020:default-3".to_string()).unwrap());

        let fs = fs::read_to_string(home + &"/.rusty-forest/trees.conf");
        let fs = match fs {
        Err(_) => { String::new() }
        Ok(x)  => { x }
        };
    
        for tree_str in fs.lines() {
            let tree = Tree::import_tree(tree_str.to_string());
            match tree {
            Ok(x) => { trees.push(x); }
            Err(x) => { println!("Failed to load tree: {}", x); }
            };
        }
        
        TreeCollection {
            collection: trees
        }
    }

    /// Add a tree to the tree collection, and be careful to not add a duplicate tree.
    /// If duped is true, the tree will be renamed to not collide with other trees.
    pub fn add_tree(&mut self, tree: String, duped: bool) -> Result<(), String> {
        let mut tree = Tree::import_tree(tree)?;
       

        let mut cnt = 0;
        let mut failed = true;
        while failed {
            failed = false;

            let new_name = if cnt == 0 {
                tree.name.clone()
            } else {
                tree.name.clone() + &format!("-{}", cnt)
            };
            
            for other_tree in &self.collection {
                if other_tree.name == new_name {
                    if !duped {
                        return Err("Duplicate name tree exists".to_string());
                    } else {
                        failed = true;
                    }
                }
            }
            if failed {
                cnt = cnt + 1;
            }
        }

        if cnt != 0 {
            tree.name = tree.name + &format!("-{}", cnt);
        }

        self.collection.push(tree);
        Ok(())
    }

    /// Save all trees in `~/.rusty-forest/trees.conf`.
    pub fn save(&self) -> Result<(), String> {
        let home = std::env::var("HOME");

        let home = match home {
        Ok(x) => { x }
        Err(x) => { return Err(format!("{}", x)); }
        };
        
        let mut file = File::create(home + &"/.rusty-forest/trees.conf").unwrap();

        for tree in &self.collection {
            if tree.name != "default" && tree.name != "default-2" && tree.name != "default-3" {
                file.write_all((tree.to_string() + &"\n").as_bytes()).unwrap();
            }
        }
        
        Ok(())
    }
}

/// Check the directories that hold the saved data. Create them if they do not exist.
fn check_directories() -> Result<(), String> {
    let home = std::env::var("HOME");

    let home = match home {
    Ok(x) => { x }
    Err(x) => { return Err(format!("{}", x)); }
    };

    let res = DirBuilder::new()
        .recursive(true)
        .create(home + &"/.rusty-forest");
    
    if let Err(x) = res {
        return Err(format!("{}", x).to_string());
    };

    Ok(())
}

/// This holds all the data of an already grown tree. The data refers to 
/// how much the tree has grown, the tree itself, its label and the date 
/// it was grown.
///
/// The format is `<duration as hh:mm>/<label>/<timstamp>/<formatted-tree>`.
#[derive(Debug)]
pub struct GrownTree {
    pub duration: u64,
    pub tree: Tree,
    pub label: String,
    pub timestamp: i64,
}

impl FromStr for GrownTree {
    type Err = String;

    /// Parse this struct from a string that respects the format.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens: Vec<&str> = s.split('/').collect();
        

        if tokens.len() != 4 {
            return Err("Failed to parse stats: wrong number of tokens".to_string());
        }

        let duration = GrowthTime::from_str(tokens[0])?;        
        let duration: u64 = duration.h * 60 + duration.m;
        let label = tokens[1].to_string();
        let timestamp = match i64::from_str(tokens[2]) {
        Ok(x) => { x }
        Err(x) => { return Err(format!("Failed to parse stats: {}", x)); }
        };
        
        let tree = Tree::import_tree(tokens[3].to_string())?;

        Ok( GrownTree {
            duration,
            tree,
            label,
            timestamp
        } )
    }
}

/// Load all the grown trees from `~/.rusty-forest/stats.conf`.
pub fn get_stats() -> Result<Vec<GrownTree>, String> {
    check_directories().expect("Failed to check directories");
    
    let home = std::env::var("HOME");

    let home = match home {
    Ok(x) => { x }
    Err(_) => { return Ok(Vec::new()); }
    };
    
    let mut trees: Vec<GrownTree> = Vec::new();
    let fs = fs::read_to_string(home + &"/.rusty-forest/stats.conf");
    let fs = match fs {
    Err(_) => { String::new() }
    Ok(x)  => { x }
    };
    
    for line in fs.lines() {
        let tree = GrownTree::from_str(line);
        match tree {
        Ok(x) => { trees.push(x); }
        Err(x) => { println!("Failed to load tree: {}", x); }
        }
    }

    Ok(trees)
}

