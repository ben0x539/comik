#![windows_subsystem = "windows"]

mod app;
mod ui;
mod providers;
mod resource;

fn main() -> Result<(), String> {
    let mut app = app::App::new();

    app.run()
}
