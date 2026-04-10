#![windows_subsystem = "windows"]

use std::error::Error;

mod tree;
mod render;
mod app;

fn main() -> Result<(), Box<dyn Error>> {
    app::run()
}