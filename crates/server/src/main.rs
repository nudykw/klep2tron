use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use std::time::Duration;

fn main() {
    App::new()
        // Headless mode для сервера: 60 тиков в секунду
        .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        ))))
        .add_systems(Startup, setup_server)
        .run();
}

fn setup_server() {
    println!("Kleptotron Server запущен!");
}
