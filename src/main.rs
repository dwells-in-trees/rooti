#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;

mod tree;
mod render;
mod app;
mod diagnostics;

fn main() -> Result<(), Box<dyn Error>> {
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("{info}\n{}", std::backtrace::Backtrace::capture());
        std::fs::write("panic.txt", msg).ok();
    }));
    app::run()
}