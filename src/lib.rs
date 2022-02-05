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
//!    // Timers are Bevy systems, and thus can be closures. 
//!    timers.after(5, (move || {
//!        println!("timer has gone off!");
//!    }).system());
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
use bevy::ecs::Stage;
use bevy::prelude::*;
use std::mem::MaybeUninit;
use std::mem;

const MAX_INTERVAL: usize = 64;

struct TimingWheel {
    current_tick: usize,
    ring:         [Vec<(usize, Box<dyn Stage>)>; MAX_INTERVAL],
}

impl Default for TimingWheel {
    fn default() -> Self {
        let mut empty = MaybeUninit::<[Vec<_>; MAX_INTERVAL]>::uninit();
        let p: *mut Vec<Box<dyn Stage>> = unsafe { mem::transmute(&mut empty) };
        for i in 0..MAX_INTERVAL {
            unsafe {
                p.add(i).write(vec![]);
            }
        }
        TimingWheel {
            current_tick: 0,
            ring:         unsafe { empty.assume_init() },
        }
    }
}

impl TimingWheel {
    /// Insert the timer into the wheel. 
    fn schedule(&mut self, offset: usize, ticks: usize, timer: Box<dyn Stage>) {
        let index = (self.current_tick + offset) % MAX_INTERVAL;
        self.ring[index].push((ticks, timer));
    }

    /// Return all the timers that execute on the current tick, and more the clock
    /// forward one. 
    fn tick(&mut self) -> Vec<(usize, Box<dyn Stage>)> {
        let timers = mem::take(&mut self.ring[self.current_tick]);
        self.current_tick = (self.current_tick + 1) % MAX_INTERVAL;
        timers
    }
}

/// A Bevy resource that allows for the scheduling of tick based timers. 
#[derive(Default)]
pub struct Timers {
    /// One frame at 120 fps.
    level_0: TimingWheel,
    level_1: TimingWheel,
    level_2: TimingWheel,
    level_3: TimingWheel,
    // TODO: Add more levels (if you want to). 
}

impl Timers {
    /// Schedule a timer to occur after the given number of ticks have elapsed. 
    pub fn after<S>(&mut self, after: usize, timer: S)
    where
        S: System<In = (), Out = ()>
    {
        let timer = Box::new(SystemStage::from(timer));
        let level = if after == 0 {
            0
        } else {
            (63 - after.leading_zeros()) / 6
        };
        match level {
            0 => self.level_0.schedule(after, after, timer),
            1 => self.level_1.schedule(after >> 6 - 1, after, timer),
            2 => self.level_2.schedule(after >> 12 - 1, after, timer),
            3 => self.level_3.schedule(after >> 18 - 1, after, timer),
            _ => panic!("timer interval too long"),
        }
    }

    /// Schedule a timer to occur right now.
    pub fn now<S>(&mut self, timer: S)
    where
        S: System<In = (), Out = ()>
    {
        self.after(0, timer);
    }

    fn tick(&mut self) -> Vec<Box<dyn Stage>> {
        // Surely there is a better way to do this.
        if self.level_0.current_tick == 63 {
            if self.level_1.current_tick == 63 {
                if self.level_2.current_tick == 63 {
                    for (tick, item) in self.level_3.tick() {
                        self.level_2.schedule(tick + 1, tick, item);
                    }
                }
                for (tick, item) in self.level_2.tick() {
                    self.level_1.schedule(tick + 1, tick, item);
                }
            }
            for (tick, item) in self.level_1.tick() {
                self.level_0.schedule(tick + 1, tick, item);
            }
        }
        self.level_0.tick().into_iter().map(|(_, x)| x).collect()
    }
}

#[derive(Default)]
struct RunTimers {
    curr_timers: Vec<Box<dyn Stage>>,
}

impl Stage for RunTimers {
    fn initialize(&mut self, world: &mut World, resources: &mut Resources) {
        let mut timers = resources.get_mut::<Timers>().unwrap().tick();
        for timer in &mut timers {
            timer.initialize(world, resources);
        }
        self.curr_timers = timers;
    }

    fn run(&mut self, world: &mut World, resources: &mut Resources) {
        let timers = mem::take(&mut self.curr_timers);
        for mut timer in timers {
            timer.run(world, resources);
        }
    }
}

/// A Bevy plugin that adds the [Timers] resource and a scheduler to execute timers each
/// game update.
pub struct TimerPlugin;

impl Plugin for TimerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(Timers::default())
            .add_stage("run timers", RunTimers::default());
    }
}
