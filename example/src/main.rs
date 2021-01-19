use bevy::prelude::*;
use bevy_tick_timers::{TimerPlugin, Timers};

fn add_timer(
    mut timers: ResMut<Timers>,
) {
    // Timers can be closures.
    timers.after(5, (move || {
        println!("timer has gone off!");
    }).system());
}

fn mark_ticks() {
    println!("tick");
}

fn main() {
    println!("starting up");
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(TimerPlugin)
        .add_startup_system(add_timer.system())
        .add_system(mark_ticks.system())
        .run();
}

