use rand::Rng;
use raylib::prelude::*;
use rlua::{prelude::*, StdLib, Variadic};

/// A Grid is a collection of Cells.
#[derive(Debug, Clone)]
pub struct Grid {
    pub width: i32,
    pub height: i32,
    pub cells: Vec<Vec<Cell>>,
    pub cell_prescriptors: CellPrescriptors,
}

impl LuaUserData for Grid {
    fn add_methods<'lua, T: LuaUserDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("kernel", |ctx, this, (x, y): (i32, i32)| {
            let globals = ctx.globals();
            let gx = globals.get::<_, i32>("x").unwrap();
            let gy = globals.get::<_, i32>("y").unwrap();
            if let Some(cell) = this.try_get_cell(gx + x, gy + y) {
                let table = ctx.create_table().unwrap();
                table.set("state", cell.state).unwrap();
                table.set("matter", this.cell_prescriptors[cell.state as usize].matter).unwrap();
                Ok(table)
            } else {
                let table = ctx.create_table().unwrap();
                table.set("state", 255).unwrap();
                table.set("matter", 0).unwrap();
                Ok(table)
            }
        });
        methods.add_method_mut("update", |ctx, this, (x, y, state): (i32, i32, i32)| {
            let globals = ctx.globals();
            let gx = globals.get::<_, i32>("x").unwrap();
            let gy = globals.get::<_, i32>("y").unwrap();
            Ok(this.change_cell(gx + x, gy + y, state as u8).is_ok())
        });
        methods.add_method("choose", |ctx, _, ():()| {
            let (x, y) = (rand::thread_rng().gen_range(-1..2), rand::thread_rng().gen_range(-1..2));
            let table = ctx.create_table().unwrap();
            table.set("x", x).unwrap();
            table.set("y", y).unwrap();
            Ok(table)
        });
        methods.add_method_mut("copy", |ctx, this, (x, y): (i32, i32)| {
            let globals = ctx.globals();
            let gx = globals.get::<_, i32>("x").unwrap();
            let gy = globals.get::<_, i32>("y").unwrap();
            let other = if let Some(cell) = this.try_get_cell(gx + x, gy + y) {
                cell.state
            } else {
                0
            };
            Ok(this.change_cell(gx, gy, other).is_ok())
        });
        methods.add_method("cellState", |_, this, (x, y): (i32, i32)| {
            if let Some(cell) = this.try_get_cell(x, y) {
                Ok(cell.state)
            } else {
                Ok(255)
            }
        });
        methods.add_method("cellMatter", |_, this, (x, y): (i32, i32)| {
            if let Some(cell) = this.try_get_cell(x, y) {
                Ok(this.cell_prescriptors[cell.state as usize].matter)
            } else {
                Ok(0)
            }
        });
        methods.add_method_mut("swap", |ctx, this, (x, y): (i32, i32)| {
            let globals = ctx.globals();
            let gx = globals.get::<_, i32>("x").unwrap();
            let gy = globals.get::<_, i32>("y").unwrap();
            let other = if let Some(cell) = this.try_get_cell(gx + x, gy + y) {
                cell.state
            } else {
                0
            };
            this.change_cell(gx + x, gy + y, this.cells[gx as usize][gy as usize].state);
            Ok(this.change_cell(gx, gy, other).is_ok())
        });
        methods.add_method("findAll", |ctx, this, state: i32| {
            let table = ctx.create_table().unwrap();
            let gx = ctx.globals().get::<_, i32>("x").unwrap();
            let gy = ctx.globals().get::<_, i32>("y").unwrap();
            for i in gx-1..gx+2 {
                for j in gy-1..gy+2 {
                    if let Some(cell) = this.try_get_cell(i, j) {
                        if cell.state == state as u8 {
                            let table2 = ctx.create_table().unwrap();
                            table2.set("x", i-gx).unwrap();
                            table2.set("y", j-gy).unwrap();
                            table.set(table.len().unwrap() + 1, table2).unwrap();
                        }
                    }
                }
            }
            Ok(table)
        });
        methods.add_method("isAround", |ctx, this, state: Variadic<i32> | {
            let gx = ctx.globals().get::<_, i32>("x").unwrap();
            let gy = ctx.globals().get::<_, i32>("y").unwrap();
            for i in gx-1..gx+2 {
                for j in gy-1..gy+2 {
                    if let Some(cell) = this.try_get_cell(i, j) {
                        for s in state.clone().iter(){
                            if cell.state == *s as u8 {
                                return Ok(true)
                            }
                        }
                    }
                }
            }
            Ok(false)
        });
        methods.add_method("count", |ctx, this, state: Variadic<i32> | {
            let gx = ctx.globals().get::<_, i32>("x").unwrap();
            let gy = ctx.globals().get::<_, i32>("y").unwrap();
            let mut count = 0;
            for i in gx-1..gx+2 {
                for j in gy-1..gy+2 {
                    if let Some(cell) = this.try_get_cell(i, j) {
                        for s in state.clone().iter(){
                            if cell.state == *s as u8 {
                                count += 1;
                            }
                        }
                    }
                }
            }
            Ok(count)
        });
    }
}

impl Grid {
    pub fn new(width: i32, height: i32, init: Option<String>) -> (Self, Lua) {
        let mut cells = Vec::new();
        for x in 0..width {
            let mut row = Vec::new();
            for y in 0..height {
                row.push(Cell {
                    x,
                    y,
                    state: 0,
                });
            }
            cells.push(row);
        }
        let mut cell_prescriptors = Vec::new();
        cell_prescriptors.push(CellPrescriptor {
            color: raylib::color::Color::BLACK,
            update: None,
            matter: 255,
            name: "air".to_string(),
        });
        let mut to_change = Vec::new();
        for x in 0..width {
            for y in 0..height {
                to_change.push((x, y));
            }
        }
        let lua = Lua::new_with(StdLib::BASE | StdLib::MATH | StdLib::TABLE | StdLib::STRING);

        // generate seed for the lua rng
        lua.context(|ctx| {
            let globals = ctx.globals();
            globals
                .set("seed", rand::thread_rng().gen_range(0..u32::MAX))
                .unwrap();
            ctx.load("math.randomseed(seed)").exec().unwrap();
        });
        (Grid {
            width,
            height,
            cells,
            cell_prescriptors,
        }, lua)

    }
    pub fn clear(&mut self) {
        for x in 0..self.width {
            for y in 0..self.height {
                self.cells[x as usize][y as usize].state = 0;
            }
        }
    }
    pub fn try_get_cell(&self, x: i32, y: i32) -> Option<&Cell> {
        if 0 <= x && x < self.width && 0 <= y && y < self.height {
            Some(&self.cells[x as usize][y as usize])
        } else {
            None
        }
    }
    pub fn update(lua: &Lua, (x, y): (i32, i32), script: &Option<String>) {
        if let Some(script) = script {
            lua.context(|ctx| {
                let globals = ctx.globals();
                globals.set("x", x).unwrap();
                globals.set("y", y).unwrap();
                ctx.load(script).exec().unwrap();
            });
        }
    }
    /*pub fn _update(&mut self, rng: &mut ThreadRng) {
        // fill the to_change vector if it is empty
        // this is done to avoid having to iterate over the entire grid
        if self.to_change.is_empty() {
            for x in 0..self.width {
                for y in 0..self.height {
                    self.to_change.push((x, y));
                }
            }
        }

        // randomly select a cell that needs to be updated
        let (x, y) = self
            .to_change
            .swap_remove(rng.gen_range(0..self.to_change.len()));
        let cell = &self.cells[x as usize][y as usize];

        let me = &self.cell_prescriptors[cell.state as usize];

        let mut new_state: Result<u8, LuaError> = Err(LuaError::BindError);

        let mut to_update: Vec<(i32, i32, u8)> = Vec::new();

        // call the update function
        if let Some(update) = &me.update {
            // get the kernel
            let (kernel_width, kernel_height, kernel_x, kernel_y) = (3, 3, 1, 1);
            self.lua
                .context(|ctx| {
                    let globals = ctx.globals();

                    // set my coordinates
                    globals.set("x", cell.x).unwrap();
                    globals.set("y", cell.y).unwrap();

                    let update_table = ctx.create_table().unwrap();
                    globals.set("update", update_table).unwrap();

                    globals.set("state", LuaValue::Nil).unwrap();
                    globals.set("matter", cell.state).unwrap();

                    // set the kernel
                    let kernel = globals.get::<_, LuaTable>("kernel").unwrap();
                    for x in 0..kernel_width {
                        let row = ctx.create_table().unwrap();
                        for y in 0..kernel_height {
                            // if out of bounds, set to 0
                            let state = if 0 <= x as i32 + cell.x as i32 - kernel_x as i32
                                && 0 <= y as i32 + cell.y as i32 - kernel_y as i32
                                && x + cell.x - kernel_x < self.width
                                && y + cell.y - kernel_y < self.height
                            {
                                let c = &self.cells[(x + cell.x - kernel_x) as usize]
                                    [(y + cell.y - kernel_y) as usize];
                                (c.state, self.cell_prescriptors[c.state as usize].state)
                            } else {
                                (255, 0)
                            };
                            let cell_table = ctx.create_table().unwrap();
                            cell_table.set("state", state.0).unwrap();
                            cell_table.set("matter", state.1).unwrap();
                            row.set(y + 1, cell_table).unwrap();
                        }
                        kernel.set(x + 1, row).unwrap();
                    }

                    let res = ctx.load(update).exec();

                    new_state = globals.get::<_, u8>("state");

                    let mut idx = 1;
                    let update = globals.get::<_, LuaTable>("update").unwrap();
                    while let Ok(data) = update.get::<_, LuaTable>(idx) {
                        let (x, y, state): (i32, i32, i32) = (
                            data.get(1).unwrap(),
                            data.get(2).unwrap(),
                            data.get(3).unwrap(),
                        );
                        to_update.push((
                            x - 1 + cell.x - kernel_x,
                            y - 1 + cell.y - kernel_y,
                            state as u8,
                        ));
                        idx += 1;
                    }

                    res
                })
                .unwrap();
        }
        if let Ok(new_state) = new_state {
            self.change_cell(cell.x, cell.y, new_state).unwrap();
        }

        // update the cells
        for (x, y, state) in to_update {
            self.change_cell(x, y, state);
        }
    }*/
    pub fn add_state(
        &mut self,
        color: raylib::color::Color,
        update: Option<String>,
        matter_state: u8,
        name: String,
    ) {
        // todo: add the state to the lua context
        /*self.lua.context(|ctx| {
            let globals = ctx.globals();
            globals.set(name.as_str(), self.cell_prescriptors.len()).unwrap();
        });*/
        self.cell_prescriptors.push(CellPrescriptor {
            color,
            update,
            matter: matter_state,
            name,
        });
    }
    pub fn change_cell(&mut self, x: i32, y: i32, state: u8) -> Result<(), ()> {
        if x >= self.width || y >= self.height || x < 0 || y < 0 {
            return Err(());
        }
        if state >= self.cell_prescriptors.len() as u8 {
            return Err(());
        }
        self.cells[x as usize][y as usize].state = state;
        /*let (k_x, k_y, k_width, k_height) = self.cell_prescriptors[state as usize].kernel;
        self.cells[x as usize][y as usize].kernel_spec = (x-k_x);*/
        Ok(())
    }
    pub fn draw(
        &self,
        d: &mut raylib::drawing::RaylibDrawHandle,
        (x, y): (i32, i32),
        (width, height): (i32, i32),
    ) {
        d.draw_rectangle(x, y, width, height, raylib::color::Color::BLACK);

        let width_ratio = width as f64 / self.width as f64;
        let height_ratio = height as f64 / self.height as f64;

        for row in &self.cells {
            for cell in row {
                if cell.state == 0 {
                    continue;
                }

                let cell_x = (x as f64 + cell.x as f64 * width_ratio) as i32;
                let cell_y = (y as f64 + cell.y as f64 * height_ratio) as i32;
                let cell_width = (width_ratio) as i32;
                let cell_height = (height_ratio) as i32;

                let me = &self.cell_prescriptors[cell.state as usize];
                d.draw_rectangle(cell_x, cell_y, cell_width, cell_height, me.color);
            }
        }
    }
}

/// A Cell is a single unit of a Grid.
#[derive(Debug, Clone)]
pub struct Cell {
    pub x: i32,
    pub y: i32,
    pub state: u8,
}

/// A CellPrescriptor is a list of colors and their corresponding kernels for a given cell state.
type CellPrescriptors = Vec<CellPrescriptor>;

/// A CellPrescriptor is a list of colors and their corresponding kernels for a given cell state.
#[derive(Debug, Clone)]
pub struct CellPrescriptor {
    /// The color of the cell.
    pub color: raylib::color::Color,
    /// The Lua function that updates the cell.
    pub update: Option<String>,
    /// state of matter (solid, liquid, gas, custom..)
    pub matter: u8,
    /// name of the state
    pub name: String,
}