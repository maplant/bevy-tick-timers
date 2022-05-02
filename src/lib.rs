//! `bevy-tick-timers` provides a Bevy plugin that enables the use of tick based timers.
//!
//! Tick based timers are timers that operate not on real time, but on the number of state updates
//! that occur. Each state update constitutes a "tick".
//!
//! For any timer that does not update outside of a game session, a tick based timer is preferred.
//! This makes games more consistent and replayable (which also means they are easier to debug).
//!
//! # Example:
//!```no_run
//!use bevy::prelude::*;
//!use bevy_tick_timers::{TimerPlugin, Timers};
//!
//!fn add_timer(
//!    mut timers: ResMut<Timers>,
//!) {
//!    // Timers are closures that take the world as a mutable reference.
//!    timers.after(5, |_| {
//!        println!("timer has gone off!");
//!    });
//!}
//!
//!fn main() {
//!    println!("starting up");
//!    App::build()
//!        .add_plugins(DefaultPlugins)
//!        .add_plugin(TimerPlugin)
//!        .add_startup_system(add_timer.system())
//!        .run();
//!}
//!```
// use bevy::ecs::Stage;
use bevy::prelude::*;
use std::mem;
use std::mem::MaybeUninit;

const MAX_INTERVAL: usize = 64;

type BoxedSystem = Box<dyn FnOnce(&mut World) + Send + Sync>;

struct TimingWheel {
    current_tick: usize,
    ring: [Vec<(usize, BoxedSystem)>; MAX_INTERVAL],
}

impl Default for TimingWheel {
    fn default() -> Self {
        let mut empty = MaybeUninit::<[Vec<_>; MAX_INTERVAL]>::uninit();
        let p: *mut Vec<BoxedSystem> = unsafe { mem::transmute(&mut empty) };
        for i in 0..MAX_INTERVAL {
            unsafe {
                p.add(i).write(vec![]);
            }
        }
        TimingWheel {
            current_tick: 0,
            ring: unsafe { empty.assume_init() },
        }
    }
}

impl TimingWheel {
    /// Insert the timer into the wheel.
    fn schedule(&mut self, offset: usize, ticks: usize, timer: BoxedSystem) {
        self.ring[offset].push((ticks, timer));
    }

    /// Return all the timers that execute on the current tick, and more the clock
    /// forward one.
    fn tick(&mut self) -> Vec<(usize, BoxedSystem)> {
        let timers = mem::take(&mut self.ring[self.current_tick]);
        self.current_tick = (self.current_tick + 1) % MAX_INTERVAL;
        timers
    }
}

/// A Bevy resource that allows for the scheduling of tick based timers.
#[derive(Default)]
pub struct Timers {
    /// One frame at 120 fps.
    level: [TimingWheel<C, 64>; 4],
    // TODO: Add more levels (if you want to).
}

impl Timers {
    /// Schedule a timer to occur after the given number of ticks have elapsed.
    pub fn after<S>(&mut self, after: usize, timer: S)
    where
        S: FnOnce(&mut World) + Send + Sync + 'static,
    {
        let ticks = ticks
            + self.level[0].current_tick
            + (self.level[1].current_tick << 6)
            + (self.level[2].current_tick << 12)
            + (self.level[3].current_tick << 18);
        let level = if ticks == 0 {
            0
        } else {
            (63 - ticks.leading_zeros()) / 6
        };
        match level {
            0 => self.level[0].schedule(ticks, 0, timer),
            1 => self.level[1].schedule((ticks >> 6) - 1, ticks & 0b111111, timer),
            2 => self.level[2].schedule((ticks >> 12) - 1, ticks & 0b111111111111, timer),
            3 => self.level[3].schedule((ticks >> 18) - 1, ticks & 0b111111111111111111, timer),
            _ => panic!("timer interval too long"),
        }
    }

    /// Schedule a timer to occur right now.
    pub fn now<S>(&mut self, timer: S)
    where
        S: FnOnce(&mut World) + Send + Sync + 'static,
    {
        self.after(0, timer);
    }

    fn tick(&mut self) -> Vec<BoxedSystem> {
        // Surely there is a better way to do this.
        let v = self.level[0].tick().into_iter().map(|(_, x)| x).collect();
        if self.level[0].current_tick == 63 {
            for (tick, timer) in self.level[1].tick() {
                self.level[0].schedule(tick, 0, timer);
            }
            if self.level[1].current_tick == 63 {
                for (tick, timer) in self.level[2].tick() {
                    self.level[1].schedule((tick >> 6) - 1, tick & 0b111111, timer);
                }
                if self.level[2].current_tick == 63 {
                    for (tick, timer) in self.level[3].tick() {
                        self.level[2].schedule((tick >> 6) - 1, tick & 0b111111111111, timer);
                    }
                }
            }
        }
        v
    }
}

#[derive(Default)]
struct RunTimers;

impl Stage for RunTimers {
    fn run(&mut self, world: &mut World) {
        let timers = world.get_resource_mut::<Timers>().expect("Failed").tick();
        for timer in timers {
            timer(world);
        }
    }
}

/// A Bevy plugin that adds the [Timers] resource and a scheduler to execute timers each
/// game update.
pub struct TimerPlugin;

impl Plugin for TimerPlugin {
    fn build(&self, app: &mut App) {
        app.world.insert_resource(Timers::default());
        app.add_stage("run_timers", RunTimers);
    }
}
