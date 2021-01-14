use bevy::prelude::*;
use bevy_timers::{TimerPlugin, Timers};

fn add_timer(
    timers: ResMut<Timers>,
) {
    println!("tick");
}

fn main() {
    println!("starting up");
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(TimerPlugin)
        .add_system(add_timer.system())
        .run();
}

