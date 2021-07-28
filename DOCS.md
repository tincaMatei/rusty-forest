# Rusty-forest

rusty-forest is a terminal application inspired by [Forest](https://www.forestapp.cc). The 
difference is that this doesn't monitor what you do, so it works just as a "dopamine button". 
If you procrastinate while your tree is growing, nothing is going to happen, you're just 
lying to yourself.

## Installation

Run

> cargo install rusty-forest

## Usage

For quick information, you can use

> rusty-forest --help

## Subcommands

### grow

This is the core subcommand of rusty-forest. Using this will start the process of 
growing a tree. You can label this tree with other names to get better statistics.
You can set a custom duration for the tree. By doing so, you can plant more colorful 
trees.

Options:

* -d, --duration TIME
  * set a custom duration for the tree in HH:MM format. For instance, having
    "-d 01:20" means that the tree will take 1 hour and 20 minutes to grow.
    The default duration is 20 minutes.
* -l, --label LABEL
  * set a custom label for the growing tree. This is useful for instance
    if you want to monitor how much time you're spending on each activity.
    For example, if you have "-l coding", that means that this tree is dedicated 
    for coding. This is useful for stats. The default label is "standard".
* -t, --tree TREE
  * Grow a custom tree from your tree collection. TREE should be the name.
    The default tree used is called "default".
* -n, --no-display
  * do not display the growing menu, just get messages through stdout.

### import

With this command, you can add more trees to your collection, by either creating them, 
with the tree editor, or by importing from other people.

Arguments:

If you do not use `-c` or `-f`, then you should put multiple trees in their shareable
format.

Options:

* -f, --file FILE
  * Import all the trees from a file. They should be each on a separate line, 
    in their format
* -c, --create
  * Use the tree editor to create a tree. It will be directly added to the collection.
    Using this 
* -n, --name-change
  * Rename the trees if they have the same name. For instance, if there is a tree called 
    "tree", and you want to add another tree named "tree", the second one will be renamed 
    to "tree-1"
* -e, --error
  * Display error messages about loading trees in stderr

### export

With this command, you can share some of your trees with other people.

Arguments:

If you do not use `-c` or `-f`, then you should put the names of the trees you want 
to share.

Options:

* -f, --to-file FILE
  * Export the wanted trees to the given file.
* -c, --create
  * Open the tree editor and export the created tree.
* -a, --all
  * Export all the trees from your collection.

### list

Display all the trees from your collection that you can choose to grow.

Options:

* -H, --head COUNT
  * Display the first COUNT trees from your collection.
* -T, --tail COUNT
  * Display the last COUNT trees from your collection.
* -r, --random COUNT
  * display COUNT random trees from your collection.
* -n, --no-draw
  * just list the name of the trees, without actually drawing them.
* -e, --export
  * display the selected trees in an exportable format

### erase

Erase trees from your collection that you don't want to use anymore.

Arguments:

Put all the names of the trees that you want to delete from your collection.

### stats

Display stats about trees that you've grown. If you do not use -g or -G, then
this will just display the trees that you've grown.

Options:

* -g, --grid GRID
  * Display the trees in a fixed grid size. The size should be in RxC format, for 
    instance "3x4" for 3 rows and 4 columns. Additionally, you can use "whole" to
    use a grid as big as the screen.
* -G, --graph UNIT
  * Display a graph of the relevant time unit. The possible time windows are
    daily, weekly, monthly and yearly.
* -f, --filter LABEL
  * Take only the information of the trees with the given label.
* -c, --count AMOUNT
  * Take only the last AMOUNT trees that you've grown.
* -t, --time TIME
  * Get information only from a certain time period. The time period options are
    "today", "yesterday", "this-week", "this-month" and "this-year".
* -F, --format FORMAT
  * Display the dates in a custom format; the default is "%d-%m-%Y %H:%M"
