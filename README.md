# Bevy-tick-timers

Provides a [Bevy](https://bevyengine.org/) plugin for scheduling and managing tick based timers.

Tick based timers are timers that operate not on real time, but on the number of state updates
that occur. Each state update constitutes a "tick".

For any timer that does not update during game session, a tick based timer is preferred. This makes
games more consistent and replayable (which also means they are easier to debug).

# Example: 

```rust
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
```
