use bevy::prelude::*;
use bevy_tick_timers::{TimerPlugin, Timers};

fn add_timer(
    mut timers: ResMut<Timers>,
) {
    // Timers are Bevy systems, and thus can be closures. 
    timers.after(5, (move || {
        println!("timer has gone off!");
    }).system());
}

fn main() {
    println!("starting up");
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(TimerPlugin)
        .add_startup_system(add_timer.system())
        .run();
}

