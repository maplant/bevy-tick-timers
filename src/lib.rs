use bevy::ecs::Stage;
use bevy::prelude::*;
use std::mem::MaybeUninit;
use std::mem;

const MAX_INTERVAL: usize = 64;

struct TimingWheel {
    current_tick: usize,
    ring:         [Vec<Box<dyn Stage>>; MAX_INTERVAL],
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
    fn schedule(&mut self, ticks: usize, timer: Box<dyn Stage>) {
        let index = (self.current_tick + ticks) % MAX_INTERVAL;
        self.ring[index].push(timer);
    }

    /// Return all the timers that execute on the current tick, and more the clock
    /// forward one. 
    fn tick(&mut self) -> Vec<Box<dyn Stage>> {
        let timers = mem::take(&mut self.ring[self.current_tick]);
        self.current_tick = (self.current_tick + 1) % MAX_INTERVAL;
        timers
    }
}

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
            0 => self.level_0.schedule(after, timer),
            1 => self.level_1.schedule(after >> 6, timer),
            2 => self.level_2.schedule(after >> 12, timer),
            3 => self.level_3.schedule(after >> 18, timer),
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
        let mut timers = Vec::<Box<dyn Stage>>::new();
        if self.level_0.current_tick == 63 {
            if self.level_1.current_tick == 63 {
                if self.level_2.current_tick == 63 {
                    timers.extend(self.level_3.tick());
                }
                timers.extend(self.level_2.tick());
            }
            timers.extend(self.level_1.tick());
        }
        timers.extend(self.level_0.tick());
        timers
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

pub struct TimerPlugin;

impl Plugin for TimerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(Timers::default())
            .add_stage("run timers", RunTimers::default());
    }
}
