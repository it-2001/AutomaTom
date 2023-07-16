// cstring
use std::{ffi::{CStr, CString}, time::Instant};

use grid::Grid;
use rand::Rng;
use raylib::{ffi::Rectangle, prelude::*};
use reader::read_grid;
use rlua::{UserData, LightUserData};

mod grid;
mod reader;

const WIDTH: i32 = 150;

fn main() {
    let mut rng = rand::thread_rng();
    let (mut rl, thread) = raylib::init().size(800, 450).title("automaTom").build();
    let ico = Image::load_image("kamen.png");
    if let Ok(ico) = ico {
        rl.set_window_icon(&ico);
    }
    rl.set_target_fps(60);
    // read first argument as path to grid
    use std::env;
    let mut args = env::args();
    let (grid, options) = if let Some(path) = args.nth(1) {
        read_grid(&path)
    } else {
        panic!("No file specified")
    };

    let mut iterations = 1000;
    let mut running = true;
    let mut auto_adjust = true;
    let mut selected = 0;
    let mut brush_size = 1;
    /*let mut to_update = Vec::new();
    for x in 0..WIDTH {
        for y in 0..WIDTH {
            to_update.push((x, y));
        }
    }*/

    while !rl.window_should_close() {
        // get drawing context
        let mut d = rl.begin_drawing(&thread);
        // update
        // draw user input
        // chack if mouse is in grid
        // get grid userdata from lua
        grid.context(|ctx| {
            let globals = ctx.globals();
            let mut userdata = globals.get::<_, Grid>("grid").unwrap();
            
            let mouse_pos = d.get_mouse_position();
            if mouse_pos.x > 175. && mouse_pos.x < 625. && mouse_pos.y > 0. && mouse_pos.y < 450. {
                // get mouse position in grid
                let mouse_pos = (
                    ((mouse_pos.x - 175.) / 450. * WIDTH as f32) as i32,
                    ((mouse_pos.y) / 450. * WIDTH as f32) as i32,
                );
                // set cell to selected state
                if d.is_mouse_button_down(raylib::consts::MouseButton::MOUSE_LEFT_BUTTON) {
                    // change state for cells in brush
                    for x in mouse_pos.0 - brush_size..mouse_pos.0 + brush_size {
                        for y in mouse_pos.1 - brush_size..mouse_pos.1 + brush_size {
                            userdata.change_cell(x, y, selected);
                        }
                    }
                    // send back userdata
                    globals.set("grid", userdata).unwrap();
                    userdata = globals.get::<_, Grid>("grid").unwrap();
                }
            }
            // draw grid
            if running {
                for _ in 0..iterations {
                    /*if to_update.is_empty() {
                        for x in 0..WIDTH {
                            for y in 0..WIDTH {
                                to_update.push((x, y));
                            }
                        }
                    }*/
                    // randomly select cell from to_update and remove it from the list
                    let cell = (rng.gen_range(0..WIDTH), rng.gen_range(0..WIDTH));
                    //let cell = to_update.remove(index);
                    // set userdata current cell to cell
                    globals.set("x", cell.0).unwrap();
                    globals.set("y", cell.1).unwrap();
                    // get state of the cell to determine script to run
                    let state = ctx.load("grid:cellState(x, y)").eval::<i32>().unwrap();
                    // get script to run
                    let script = &userdata.cell_prescriptors[state as usize].update;
                    Grid::update(&grid, cell, script);
                }
                userdata = globals.get::<_, Grid>("grid").unwrap();
                if auto_adjust {
                    iterations += if d.get_fps() < 24 { -20 } else { 20 };
                }
            }
            d.clear_background(Color::WHITE);
            userdata.draw(&mut d, (175, 0), (450, 450));
            // draw iterations slider
            iterations = d.gui_slider(
                Rectangle {
                    x: 10.,
                    y: 40.,
                    width: 155.,
                    height: 20.,
                },
                None,
                None,
                iterations as f32,
                0.,
                10000.,
            ) as i32;
            d.draw_text(
                &format!("Iterations: {}", iterations),
                12,
                45,
                12,
                Color::BLACK,
            );
            // draw run/pause button
            let text_to_draw = if running { "Pause" } else { "Run" };
            if d.gui_button(
                Rectangle {
                    x: 10.,
                    y: 70.,
                    width: 155.,
                    height: 20.,
                },
                Some(CString::new(text_to_draw).unwrap().as_c_str()),
            ) {
                running = !running;
            }
            // draw auto adjust button
            let text_to_draw = if auto_adjust {
                "Auto Adjust: On"
            } else {
                "Auto Adjust: Off"
            };
            if d.gui_button(
                Rectangle {
                    x: 10.,
                    y: 100.,
                    width: 155.,
                    height: 20.,
                },
                Some(CString::new(text_to_draw).unwrap().as_c_str()),
            ) {
                auto_adjust = !auto_adjust;
            }
            // draw clear button
            if d.gui_button(
                Rectangle {
                    x: 10.,
                    y: 130.,
                    width: 155.,
                    height: 20.,
                },
                Some(CString::new("Clear").unwrap().as_c_str()),
            ) {
                userdata.clear();
                // send back userdata
                globals.set("grid", userdata).unwrap();
                userdata = globals.get::<_, Grid>("grid").unwrap();
            }
            // draw cell type buttons
            for (i, cell) in userdata.cell_prescriptors.iter().enumerate() {
                if d.gui_button(
                    Rectangle {
                        x: 175. + 450. + 10.,
                        y: 10. + (i as f32 * 30.),
                        width: 155.,
                        height: 20.,
                    },
                    Some(CString::new(cell.name.to_string()).unwrap().as_c_str()),
                ) {
                    selected = i as u8;
                }
            }
            // draw brush size slider
            brush_size = d.gui_slider(
                Rectangle {
                    x: 10.,
                    y: 160.,
                    width: 155.,
                    height: 20.,
                },
                None,
                None,
                brush_size as f32,
                1.,
                10.,
            ) as i32;
            d.draw_text(
                &format!("Brush Size: {}", brush_size),
                12,
                165,
                12,
                Color::BLACK,
            );
            // draw fps
            d.draw_fps(12, 12);
        });
    }
}
