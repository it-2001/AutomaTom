use raylib::prelude::*;
use rlua::Lua;
use toml::Table;

use crate::grid::Grid;

pub struct Options {
    pub table: Vec<(String, usize)>
}

pub fn read_grid(path: &str) -> (Lua, Options) {
    // read file
    let file = std::fs::read_to_string(path).expect("Failed to read file");
    let mut options = Options { table: Vec::new() };
    // parse file
    let parsed: Table = toml::from_str(&file).expect("Failed to parse file");
    // create grid
    let width = 150;
    let height = 150;
    let init = parsed.get("entry").unwrap().as_table().unwrap();
    let init = if let Some(init) = init.get("init") {
        Some(init.as_str().unwrap().into())
    } else {
        None
    };
    let (mut grid, mut lua) = Grid::new(width, height, init);
    // create states
    let states = parsed.get("cell").unwrap().as_table().unwrap();
    for (name, state) in states {
        let state = state.as_table().unwrap();
        let color = state.get("color").unwrap().as_array().unwrap();
        /*let color = match color {
            "red" => raylib::Color::RED,
            "green" => raylib::Color::GREEN,
            "blue" => raylib::Color::BLUE,
            "yellow" => raylib::Color::YELLOW,
            "orange" => raylib::Color::ORANGE,
            "purple" => raylib::Color::PURPLE,
            "brown" => raylib::Color::BROWN,
            "white" => raylib::Color::WHITE,
            "black" => raylib::Color::BLACK,
            _ => raylib::Color::BLANK,
        };*/
        let matter = state.get("state").unwrap().as_integer().unwrap() as u8;
        let update = if let Some(update) = state.get("update") {
            Some(update.as_str().unwrap().into())
        } else {
            None
        };
        grid.add_state(
            Color {
                r: (color[0].as_float().unwrap() * 255.) as u8,
                g: (color[1].as_float().unwrap() * 255.) as u8,
                b: (color[2].as_float().unwrap() * 255.) as u8,
                a: 255,
            },
            update,
            matter,
            name.to_string(),
        );
    }

    // send grid as userdata to lua
    lua.context(|lua_ctx| {
        let globals = lua_ctx.globals();
        for (i, cell) in grid.cell_prescriptors.iter().enumerate() {
            globals.set(cell.name.as_str(), i).unwrap();
        }
        globals.set("grid", grid).unwrap();
    });
    (lua, options)
}
