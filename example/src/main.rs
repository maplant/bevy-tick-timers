use bevy::prelude::*;
use bevy_tick_timers::{TimerPlugin, Timers};

fn add_timer(
    mut timers: ResMut<Timers>,
) {
    // Timers are Bevy systems, and thus can be closures. 
    timers.after(129, test);
    println!("System scheduled");
}

fn test(_world: &mut World) {
    println!("Timer has gone off");
}
    
fn main() {
    println!("starting up");
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TimerPlugin)
        .add_startup_system(add_timer.system())
        .run();
}
