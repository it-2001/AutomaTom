# AutomaTom

![icon](https://github.com/it-2001/AutomaTom/blob/master/kamen.png)

## Description

AutomaTom is an asynchronous cellular automaton engine written in Rust. It is not really useful for anything, but it is a fun project to work on. Asynchronous meaning that the cells are updated in a random order.

## Usage

Head over to the releases page and download the latest release (windows only). There is also a demo script `simulation.toml` that you can run with the engine by simply dragging it onto the executable.

## Building

This is meant for people who want to build the engine themselves. You will need to have Rust installed. You can get it from [here](https://www.rust-lang.org/tools/install). Once you have Rust installed, you can clone the repository and run `cargo build --release` to build the engine. The executable will be in `target/release/`. Not sure if you need raylib installed, but if you do, you can get it from [here](https://www.raylib.com/).

## Scipting

The engine uses both toml and Lua for scripting. The toml file is used to configure the engine and the Lua file is used to define the rules of the simulation. The toml file is structured as follows:

```toml
[entry]
init = '''
-- Here is where you can define the initial state of the simulation
-- sadly this is not yet implemented but is required for the engine to run
'''

[cells.sand]
color = [1.0, 1.0, 0.0] # RGB
state = 1 # The state of the matter (indestructible, solid, liquid, gas, plasma, etc.)
update = '''
-- Here is where you can define the rules for updating the cell
-- This is where you can use the Lua API to interact with the simulation
-- The API is documented below
'''
```

### Lua API

The Lua API is used to interact with the simulation.

to access the API, you can use the `grid` global variable.

method | description
--- | ---
`grid:kernel(x, y)` | returns the cell relative to the current cell
`grid:update(x, y, state)` | updates the cell relative to the current cell
`grid:choose(x, y)` | returns a random cell relative to the current cell
`grid:copy(x, y)` | sets the current cell to the cell relative to the given position
`grid:cellState(x, y)` | returns the state of the cell at the given position
`grid:cellMatter(x, y)` | returns the matter of the cell at the given position
`grid:swap(x, y)` | swaps the current cell with the cell relative to the current cell
`grid:findAll(state...)` | returns a list of all cells with the given state(s)
`grid:isAround(state...)` | returns true if any of the cells around the current cell have the given state(s)
`grid:count(state...)` | returns the number of cells around the current cell with the given state(s)

Each cell type is also a global variable. For example, if you have a cell type called `sand`, you can access it with the `sand` global variable. So if you want to know how much sand is around the current cell, you can use `grid:count(sand)`.

The random number generator is initialized and can be accesesed with `math.random(min, max)`.

The position of the current cell can be accessed with the `x` and `y` global variables. Normaly you don't need to use these, but they can be useful for debugging.

## Note

All cells are initialized to the `air` cell type. It can not be changed and and it is not mutable. It is also not possible to create new cell types at runtime. This is because the engine is designed to be as fast as possible. If you want to create a new cell type, you will have to add it to the source code and recompile the engine.

> That's about it. Have fun!