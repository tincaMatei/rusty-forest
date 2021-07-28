//! # Rusty-forest
//!
//! rusty-forest is a terminal application inspired by [Forest](https://www.forestapp.cc). The 
//! difference is that this doesn't monitor what you do, so it works just as a "dopamine button". 
//! If you procrastinate while your tree is growing, nothing is going to happen, you're just 
//! lying to yourself.
//!
//! ## Installation
//!
//! Run
//!
//! > cargo install rusty-forest
//!
//! ## Usage
//!
//! For quick information, you can use
//!
//! > rusty-forest --help
//!
//! ## Subcommands
//!
//! ### grow
//!
//! This is the core subcommand of rusty-forest. Using this will start the process of 
//! growing a tree. You can label this tree with other names to get better statistics.
//! You can set a custom duration for the tree. By doing so, you can plant more colorful 
//! trees.
//!
//! Options:
//!
//! * -d, --duration TIME
//!   * set a custom duration for the tree in HH:MM format. For instance, having
//!     "-d 01:20" means that the tree will take 1 hour and 20 minutes to grow.
//!     The default duration is 20 minutes.
//! * -l, --label LABEL
//!   * set a custom label for the growing tree. This is useful for instance
//!     if you want to monitor how much time you're spending on each activity.
//!     For example, if you have "-l coding", that means that this tree is dedicated 
//!     for coding. This is useful for stats. The default label is "standard".
//! * -t, --tree TREE
//!   * Grow a custom tree from your tree collection. TREE should be the name.
//!     The default tree used is called "default".
//! * -n, --no-display
//!   * do not display the growing menu, just get messages through stdout.
//!
//! ### import
//!
//! With this command, you can add more trees to your collection, by either creating them, 
//! with the tree editor, or by importing from other people.
//!
//! Arguments:
//!
//! If you do not use `-c` or `-f`, then you should put multiple trees in their shareable
//! format.
//!
//! Options:
//!
//! * -f, --file FILE
//!   * Import all the trees from a file. They should be each on a separate line, 
//!     in their format
//! * -c, --create
//!   * Use the tree editor to create a tree. It will be directly added to the collection.
//!     Using this 
//! * -n, --name-change
//!   * Rename the trees if they have the same name. For instance, if there is a tree called 
//!     "tree", and you want to add another tree named "tree", the second one will be renamed 
//!     to "tree-1"
//!
//! ### export
//!
//! With this command, you can share some of your trees with other people.
//!
//! Arguments:
//!
//! If you do not use `-c` or `-f`, then you should put the names of the trees you want 
//! to share.
//!
//! Options:
//!
//! * -f, --to-file FILE
//!   * Export the wanted trees to the given file.
//! * -c, --create
//!   * Open the tree editor and export the created tree.
//! * -a, --all
//!   * Export all the trees from your collection.
//!
//! ### list
//!
//! Display all the trees from your collection that you can choose to grow.
//!
//! Options:
//!
//! * -H, --head COUNT
//!   * Display the first COUNT trees from your collection.
//! * -T, --tail COUNT
//!   * Display the last COUNT trees from your collection.
//! * -r, --random COUNT
//!   * display COUNT random trees from your collection.
//! * -n, --no-draw
//!   * just list the name of the trees, without actually drawing them.
//! * -e, --export
//!   * display the selected trees in an exportable format
//!
//! ### erase
//!
//! Erase trees from your collection that you don't want to use anymore.
//!
//! Arguments:
//!
//! Put all the names of the trees that you want to delete from your collection.
//!
//! ### stats
//!
//! Display stats about trees that you've grown. If you do not use -g or -G, then
//! this will just display the trees that you've grown.
//!
//! Options:
//!
//! * -g, --grid GRID
//!   * Display the trees in a fixed grid size. The size should be in RxC format, for 
//!     instance "3x4" for 3 rows and 4 columns. Additionally, you can use "whole" to
//!     use a grid as big as the screen.
//! * -G, --graph UNIT
//!   * Display a graph of the relevant time unit. The possible time windows are
//!     daily, weekly, monthly and yearly.
//! * -f, --filter LABEL
//!   * Take only the information of the trees with the given label.
//! * -c, --count AMOUNT
//!   * Take only the last AMOUNT trees that you've grown.
//! * -t, --time TIME
//!   * Get information only from a certain time period. The time period options are
//!     "today", "yesterday", "this-week", "this-month" and "this-year".
//! * -F, --format FORMAT
//!   * Display the dates in a custom format; the default is "%d-%m-%Y %H:%M"

extern crate getopts;
use getopts::Options;
use std::env;
use std::fs::{self, OpenOptions};
use crate::tree::{TreeCollection, Tree, get_stats};
use crate::editor::run_tree_editor;
use crate::grow::{GrowthTime, grow_tree};
use std::io::{Write, stdout};
use std::str::FromStr;
use std::cmp;
use termion::{color, terminal_size};
use rand::thread_rng;
use rand::seq::SliceRandom;
use regex::Regex;
use chrono::{Local, TimeZone, Datelike, DurationRound, Duration, DateTime};
use itertools::Itertools;

pub mod tree;
pub mod editor;
pub mod display;
pub mod grow;

/// rusty-forest version number.
const VERSION: &str = "0.1.1";

/// Print the entire usage of the program.
fn print_whole_usage(program: &str, opts: Options) {
    let brief = format!(r"
Usage: {} [OPTIONS]
       {} [OPTIONS] COMMAND

Commands: grow          grow a tree
          import        import a tree from other people
          export        export trees to share with other people
          list          list all created/imported trees
          stats         display stats about all grown trees
          erase         erase a tree from the collection", program, program);

    print!("{}", opts.usage(&brief));
}

/// Print the program version.
fn print_version(program: &str) {
    println!("{} {}", program, VERSION);
}

/// Build the basic opts for the program.
fn build_default_opts() -> Options {
    let mut opts = Options::new();
    opts.optflag("h", "help", "display the help menu");
    opts.optflag("v", "version", "display the version number");
    opts
}

/// Print the instructions for the import subprogram.
fn print_import_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} import TREE", program);
    
    print!("{}", opts.usage(&brief));
}

/// Build the opts for the import subprogram.
fn build_import_opts() -> Options {
    let mut opts = Options::new();

    opts.optflag("h", "help", "display this help menu");
    opts.optopt("f", "file", "import trees from the file; using this, TREE should be omitted", "FILE");
    opts.optflag("c", "create", "open the tree editor; using this, TREE should be omitted");
    opts.optflag("n", "name-change", "change names to avoid duplicate names; without this, duplicate names are ignored");
    opts.optflag("e", "error", "display error messages when importing trees");
    opts
}

/// Print the instructions for the list subprogram.
fn print_list_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} list", program);

    print!("{}", opts.usage(&brief));
}

/// Build the opts for the list subprogram.
fn build_list_opts() -> Options {
    let mut opts = Options::new();

    opts.optflag("h", "help", "display this help menu");
    opts.optopt("H", "head", "display the first N trees", "COUNT");
    opts.optopt("T", "tail", "display the last N trees", "COUNT");
    opts.optopt("r", "random", "display N random trees", "COUNT");
    opts.optflag("n", "no-draw", "do not draw the trees themselves");
    opts.optflag("e", "export", "display the trees in an exportable format");
    opts
}

/// Print the instructions for the list subprogram.
fn print_export_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} export NAME", program);

    print!("{}", opts.usage(&brief));
} 

/// Build the opts for the export subprogram.
fn build_export_opts() -> Options {
    let mut opts = Options::new();
    
    opts.optflag("h", "help", "display this help menu");
    opts.optopt("f", "to-file", "export trees to file", "FILE");
    opts.optflag("c", "create", "open the tree editor; using this, NAME should be omitted");
    opts.optflag("a", "all", "export all the trees");
    opts
}

/// Print the instructions for the grow subprogram.
fn print_grow_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} grow", program);
    print!("{}", opts.usage(&brief));
}

/// Build the opts for the grow subprogram.
fn build_grow_opts() -> Options {
    let mut opts = Options::new();
    
    opts.optflag("h", "help", "display this help menu");
    opts.optopt("d", "duration", "set custom growth time; format is H:M; if omitted, the default is 20m", "TIME");
    opts.optopt("l", "label", "set a custom label for this tree", "LABEL");
    opts.optopt("t", "tree", "grow a custom tree", "TREE");
    opts.optflag("n", "no-display", "do not display the growing menu");

    opts
}

/// Print the instructions for the stats subprogram.
fn print_stats_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} stats", program);
    print!("{}", opts.usage(&brief));
}

/// Build the opts for the stats subprogram.
fn build_stats_opts() -> Options {
    let mut opts = Options::new();

    opts.optflag("h", "help", "display this help menu");
    opts.optopt("g", "grid", "display the trees in a fixed grid size; the grid size is RxC format", "GRID");
    opts.optflag("n", "no-forest", "do not display the trees in a grid");
    opts.optopt("G", "graph", "display a graph of the relevant time unit (DAILY, WEEKLY, MONTHLY, YEARLY)", "UNIT");
    opts.optopt("f", "filter", "filter grown trees by label", "LABEL");
    opts.optopt("c", "count", "display only the most recent trees", "AMOUNT");
    opts.optopt("t", "time", "get information only from a certain time period", "TIME");
    opts.optopt("F", "format", "display dates in a custom format; default is %d-%m-%Y %H:%M", "FORMAT");

    opts
}

/// Print the instructions for the erase subprogram.
fn print_erase_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} erase TREES", program);
    print!("{}", opts.usage(&brief));
}

/// Build the opts for the erase program.
fn build_erase_opts() -> Options {
    let mut opts = Options::new();

    opts.optflag("h", "help", "display this help menu");

    opts
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let default_opts = build_default_opts();

    if args.len() < 2 {
        print_whole_usage(&program, default_opts);
        return;
    }

    let subprogram = args[1].clone();

    let mut trees = TreeCollection::load();

    match subprogram.as_str() {
    "grow" => {
        let opts = build_grow_opts();

        let matches = opts.parse(&args[2..]).unwrap();

        if matches.opt_present("h") {
            print_grow_usage(&program, opts);
            return;
        }

        let nogui = matches.opt_present("n");

        let duration_str = match matches.opt_str("d") {
        Some(x) => { x }
        None    => { "00:20".to_string() }
        };
        
        let growth_time = GrowthTime::from_str(&duration_str);

        let growth_time = match growth_time {
        Ok(x) => { x }
        Err(x) => {
            println!("{}", x);
            return;
        }
        };

        let label = match matches.opt_str("l") {
        Some(x) => { x }
        None    => { "standard".to_string() }
        };

        let regex = Regex::new("^[-_ a-zA-Z0-9]+$").unwrap();
        if !regex.is_match(&label) {
            println!("Illegal characters in label name");
            std::process::exit(1);
        }

        let tree_name = match matches.opt_str("t") {
        Some(x) => { x }
        None    => { "default-1".to_string() }
        };
    
        let mut chosen_tree: Option<Tree> = None;

        for tree in trees.collection {
            if tree.name == tree_name {
                chosen_tree = Some(tree);
            }
        }

        let chosen_tree = match chosen_tree {
        Some(x) => { x }
        None    => {
            println!("Failed to find chosen tree!");
            return;
        }
        };
        
        let tree_cost = chosen_tree.cost();
        if growth_time.to_min() < tree_cost {
            println!("This tree is too expsensive. It needs more time ({:02}:{:02}) to grow.", tree_cost / 60, tree_cost % 60);
            return;
        }

        grow_tree(chosen_tree, label, growth_time, nogui);
    }
    "import" => { // TODO: display loaded trees data
        let opts = build_import_opts();
        
        let matches = opts.parse(&args[2..]).unwrap();
        
        // Display help menu
        if matches.opt_present("h") {
            print_import_usage(&program, opts);
            return;
        }
        
        let duped = matches.opt_present("n");
        
        let write_errors = matches.opt_present("e");

        // get the content to import
        let content = if let Some(x) = matches.opt_str("f") {
            let fs = fs::read_to_string(x).unwrap();

            fs.lines().map(|x| { x.to_string() }).collect()
        } else if matches.opt_present("c") {
            vec![run_tree_editor().to_string()]
        } else {
            if matches.free.is_empty() {
                print_import_usage(&program, opts);
                return;
            }
            matches.free
        };
        
        let mut loaded: usize = 0;
        let mut tree_name: Vec<String> = Vec::new();

        for tree in content {
            let res = trees.add_tree(tree.clone(), duped);
            match res {
            Err(x) => {
                if write_errors {
                    eprintln!("Failed to add tree: {}", x);
                }
            }
            Ok(x) => {
                tree_name.push(x.name);
                loaded = loaded + 1;
            }
            }
        }
        
        println!("Loaded {} trees in total:", loaded);
        for new_name in tree_name {
            println!("{}", new_name);
        }

        trees.save()
            .expect("Failed to save trees");
    }
    "export" => {
        let opts = build_export_opts();

        let matches = opts.parse(&args[2..]).unwrap();

        if matches.opt_present("h") {
            print_export_usage(&program, opts);
            return;
        }
    
        let to_export_trees = matches.free.clone();
        let export_all = matches.opt_present("a");

        let exported = if matches.opt_present("c") { // we should use the tree editor
            vec![run_tree_editor().to_string()]
        } else { // we should search for the tree
           if export_all {
                let mut found: Vec<String> = Vec::new();
                for tree in &trees.collection {
                    found.push(tree.to_string());
                }
                found
            } else {
                if to_export_trees.is_empty() {
                    print_export_usage(&program, opts);
                    return;
                }

                let mut res = Vec::new();
                for export_tree in to_export_trees {
                    let mut found: Option<String> = None;
                    for tree in &trees.collection {
                        if tree.name == export_tree {
                            found = Some(tree.to_string());
                        }
                    }
                
                    match found {
                    Some(x) => {
                        res.push(x);
                    }
                    None => {}
                    }
                }
                res
            }
        };
    
        match matches.opt_str("f") {
        Some(file_name) => {
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(file_name);

            let mut file_res = match file {
            Err(x) => { 
                println!("Error exporting to file: {}", x);
                std::process::exit(1);
            }
            Ok(x)  => { x }
            };
        
            for tree in exported {
                file_res.write_all((tree + &"\n").as_bytes())
                    .expect("Failed to write data in file");
            }
        }
        None => {
            for tree in exported {
                println!("{}", tree);
            }
        }
        }
    }
    "list" => {
        let opts = build_list_opts();

        let matches = opts.parse(&args[2..]).unwrap();
        
        if matches.opt_present("h") {
            print_list_usage(&program, opts);
            return;
        }
    
        let draw_trees = !matches.opt_present("n");
        let exportable = matches.opt_present("e");
        
        let mut head = match matches.opt_str("H") {
        Some(x) => { 
            match x.parse::<usize>() {
            Ok(val) => { val }
            Err(x) => { 
                println!("Failed to list string: head argument parsing error: {}", x);
                std::process::exit(1);
            }
            }
        }
        None =>    { trees.collection.len() }
        };

        let tail = match matches.opt_str("T") {
        Some(x) => { 
            match x.parse::<usize>() {
            Ok(val) => { val }
            Err(x) => { 
                println!("Failed to list string: tail argument parsing error: {}", x);
                std::process::exit(1);
            }
            }
        }
        None =>    { trees.collection.len() }
        };
        
        let mut trees_order: Vec<usize> = (0..trees.collection.len()).collect();
       
        match matches.opt_str("r") {
        Some(x) => { 
            head = match x.parse::<usize>() {
            Ok(val) => { val }
            Err(x) => {
                println!("Failed to list string: random parsing argument error: {}", x);
                std::process::exit(1);
            }
            };
            let mut rng = thread_rng();
            trees_order.shuffle(&mut rng);
        }
        _ => {}
        };


        for cnt in 0..trees.collection.len() {
            if cnt < head && cnt >= trees_order.len() - tail {
                let tree = &trees.collection[trees_order[cnt]];
                if exportable {
                    println!("{}", tree.to_string());
                } else {
                    println!("{}) {}", trees_order[cnt] + 1, tree.name);
                    if draw_trees {
                        for l in 0..5 {
                            for c in 0..5 {
                                tree.display_symbol(l, c);
                            }
                            
                            write!(stdout(), "{}{}\n", color::Bg(color::Reset), color::Fg(color::Reset))
                                .expect("Failed to write");
                        }
                    }
                }
            }
        }
    }
    
    "stats" => {
        let opts = build_stats_opts();
        let matches = opts.parse(&args[2..]).unwrap();
        
        if matches.opt_present("h") {
            print_stats_usage(&program, opts);
            return;
        }
        
        let mut stats = match get_stats() {
        Ok(x) => { x }
        Err(x) => { println!("{}", x); return; }
        };

        match matches.opt_str("f") {
        Some(label) => {
            stats.retain(|x| { x.label == label } )
        }
        None => {}
        }

        match matches.opt_str("t") {
        Some(t) => {
            let now = Local::now();
            
            t.to_lowercase();

            match t.as_str() {
            "today" => {
                stats.retain(|tree| {
                    let date = Local.timestamp(tree.timestamp, 0);
                    date.num_days_from_ce() == now.num_days_from_ce()
                });
            }
            "yesterday" => {
                stats.retain(|tree| {
                    let date = Local.timestamp(tree.timestamp, 0);
                    date.num_days_from_ce() + 1 == now.num_days_from_ce()
                })
            }
            "this-week" => {
                stats.retain(|tree| {
                    let date = Local.timestamp(tree.timestamp, 0);
                    date.iso_week().year() == now.iso_week().year() &&
                    date.iso_week().week() == now.iso_week().week()
                })
            }
            "this-month" => {
                stats.retain(|tree| {
                    let date = Local.timestamp(tree.timestamp, 0);
                    date.year()  == now.year() &&
                    date.month() == date.month()
                })
            }
            "this-year" => {
                stats.retain(|tree| {
                    let date = Local.timestamp(tree.timestamp, 0);
                    date.year() == now.year()
                })
            }
            _ => {
                println!("Unknown time period");
                return;
            }
            }
        }
        None => {}
        }

        match matches.opt_str("c") {
        Some(x) => {
            let count = x.parse::<usize>();

            let count = match count {
            Ok(x) => { x }
            Err(x) => {
                println!("Failed to parse count argument: {}", x);
                return;
            }
            };
        
            if count < stats.len() {
                stats.rotate_right(count);
                stats.truncate(count);
            }
        }
        None => {}
        }
        
        match matches.opt_str("g") {
        Some(x) => {
            x.to_lowercase();

            let (n, m) = if x == "whole" {
                // Try to make the grid as big as possible
                let (width, height) = terminal_size().unwrap();
                let (width, height) = (width as usize, height as usize);

                (height / 6, (width + 1) / 6)
            } else {
                let numbers: Vec<&str> = x.split("x").collect();
                
                if numbers.len() != 2 {
                    println!("Invalid grid size format");
                    return;
                }
            
                let n = numbers[0].parse::<usize>();
                let n = match n {
                Ok(n)  => { n }
                Err(x) => {
                    println!("Invalid grid size: {}", x);
                    return;
                }
                };
                
                let m = numbers[1].parse::<usize>();
                let m = match m {
                Ok(m)  => { m }
                Err(x) => {
                    println!("Invalid grid size: {}", x);
                    return;
                }
                };
                
                (n, m)
            };
            
            let mut grid_pos: Vec<(usize, usize)> = Vec::new();
            let mut grid: Vec<Vec<Option<&Tree>>> = vec![vec![None; m]; n];

            for i in 0..n {
                for j in 0..m {
                    grid_pos.push((i, j));
                }
            }
            
            let mut rng = thread_rng();
            grid_pos.shuffle(&mut rng);

            for (pos, tree) in stats.iter().enumerate() {
                if pos < grid_pos.len() {
                    grid[grid_pos[pos].0][grid_pos[pos].1] = Some(&tree.tree);
                }
            }

            for i in 0..6*n-1 {
                for j in 0..6*m-1 {
                    if i % 6 == 5 {
                        if j % 6 == 5 {    
                            write!(stdout(), "+").expect("Failed to write");
                        } else {
                            write!(stdout(), "-").expect("Failed to write");
                        }
                    } else if j % 6 == 5 {
                        write!(stdout(), "|").expect("Failed to write");
                    } else {
                        let tree_line = i / 6;
                        let tree_col  = j / 6;
                        
                        match grid[tree_line][tree_col] {
                        Some(tree) => { tree.display_symbol(i % 6, j % 6); }
                        None => {       write!(stdout(), " ").expect("Failed to write"); }
                        }

                        write!(stdout(), "{}", termion::color::Fg(termion::color::Reset))
                            .expect("Failed to write");
                        write!(stdout(), "{}", termion::color::Bg(termion::color::Reset))
                            .expect("Failed to write");
                    }
                }
                write!(stdout(), "\n").expect("Failed to write");
            }

            return;
        }
        None => {}
        }
        
        match matches.opt_str("G") {
        Some(time_option) => {
            let (width, height) = terminal_size().unwrap();
            let (width, height) = (width as usize, height as usize);

            let cnt_strips = (height - 1) / 3;
            
            let (mut strips, mut last_time) = match time_option.as_str() {
            "daily"   => {
                 let mut data_grouped: Vec<(DateTime<Local>, u64)> = Vec::new();
                 for(key, tree) in &stats.into_iter().group_by(|tree| Local.timestamp(tree.timestamp, 0).duration_trunc(Duration::days(1)).unwrap()) {
                    data_grouped.push((key, tree.map(|tree| tree.duration).sum()));
                 }

                 (data_grouped, Local::now().duration_trunc(Duration::days(1)).unwrap())
            }
            "weekly"  => {
                 let mut data_grouped: Vec<(DateTime<Local>, u64)> = Vec::new();
                 for(key, tree) in &stats.into_iter().group_by(|tree| Local.timestamp(tree.timestamp, 0).duration_trunc(Duration::weeks(1)).unwrap()) {
                    data_grouped.push((key, tree.map(|tree| tree.duration).sum()));
                 }

                 (data_grouped, Local::now().duration_trunc(Duration::weeks(1)).unwrap())
            }
            "monthly" => {
                 let mut data_grouped: Vec<(DateTime<Local>, u64)> = Vec::new();
                 for(key, tree) in &stats.into_iter().group_by(|tree| Local.ymd(Local.timestamp(tree.timestamp, 0).year(), Local.timestamp(tree.timestamp, 0).month(), 1)) {
                    data_grouped.push((key.and_hms(0, 0, 0), tree.map(|tree| tree.duration).sum()));
                 }

                 (data_grouped, Local.ymd(Local::now().year(), Local::now().month(), 1).and_hms(0, 0, 0))
            }
            "yearly"  => {
                 let mut data_grouped: Vec<(DateTime<Local>, u64)> = Vec::new();
                 for(key, tree) in &stats.into_iter().group_by(|tree| Local.ymd(Local.timestamp(tree.timestamp, 0).year(), 1, 1)) {
                    data_grouped.push((key.and_hms(0, 0, 0), tree.map(|tree| tree.duration).sum()));
                 }

                 (data_grouped, Local.ymd(Local::now().year(), 1, 1).and_hms(0, 0, 0))
            }
            _ => {
                println!("Unknown time frame option");
                return;
            }
            };

            let mut strips_final: Vec<(String, u64)> = Vec::new();

            for _ in 0..cnt_strips {
                let date_format = match time_option.as_str() {
                "daily" | "weekly" => { format!("{}", last_time.format("%d-%m")) }
                "monthly" => { format!("{}", last_time.format("%m")) }
                "yearly" => { format!("{}", last_time.format("%y")) }
                _ => { panic!("Unexpected case"); }
                };

                match strips.pop() {
                Some(x) => {
                    if last_time == x.0 {
                        strips_final.push((date_format, x.1));
                    } else {
                        strips_final.push((date_format, 0));
                        strips.push(x);
                    }
                }
                None => {
                    strips_final.push((date_format, 0));
                }
                }
            
                last_time = last_time - Duration::days(1);
                last_time = match time_option.as_str() {
                "daily" => { last_time }
                "weekly" => { last_time.duration_trunc(Duration::weeks(1)).unwrap() }
                "monthly" => { Local.ymd(last_time.year(), last_time.month(), 1).and_hms(0, 0, 0) }
                "yearly" => { Local.ymd(last_time.year(), 1, 1).and_hms(0, 0, 0) }
                _ => { panic!("Unexpected case"); }
                }
            }

            strips_final.reverse();
            let mut max_time = 1;
            for stat in &strips_final {
                max_time = cmp::max(max_time, stat.1);
            }

            let max_width = width - 1 - strips_final[0].0.len();

            for stat in &strips_final {
                write!(stdout(), "\n{}|", stat.0).expect("Failed to write");
                write!(stdout(), "{}", color::Bg(color::Rgb(0, 0, 0))).expect("Failed to write");
                let ammount = (max_width as u64) * stat.1 / max_time;
                for _ in 0..ammount {
                    write!(stdout(), " ").expect("Failed to write");
                }
                write!(stdout(), "{}", color::Bg(color::Reset)).expect("Failed to write");
                write!(stdout(), "\n\n").expect("Failed to write");
            }

            return;
        }
        None => {}
        }

        let format = match matches.opt_str("F") {
        Some(x) => { x }
        None    => { "%d-%m-%Y %H:%M".to_string() }
        };

        for tree in stats {
            println!("{} | {} | {:02}:{:02}", tree.label, Local.timestamp(tree.timestamp, 0).format(&format), tree.duration / 60, tree.duration % 60);
        }
    }

    "erase" => {
        let opts = build_erase_opts();
        let matches = opts.parse(&args[2..]).unwrap();
    
        if matches.opt_present("h") {
            print_erase_usage(&program, opts);
            return;
        }
        
        if matches.free.is_empty() {
            print_erase_usage(&program, opts);
            return;
        }

        for to_erase in matches.free {
            trees.collection.retain(|tree| { tree.name != to_erase } );
        }
    
        match trees.save() {
        Err(x) => { println!("Failed to save trees: {}", x); return; }
        _ => {}
        }
    }

    _ => {
        let matches = default_opts.parse(&args[1..]).unwrap();

        if matches.opt_present("h") {
            print_whole_usage(&program, default_opts);
            return;
        } else if matches.opt_present("v") {
            print_version(&program);
            return;
        }
    }
    }
}

