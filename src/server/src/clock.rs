use std::convert::TryInto;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Clock {
    aim: u128,
    average: u128,
    update_count: u128,
    last_time: Option<Instant>,
}

impl Clock {
    //This sets the UPS of the clock. This determines the duration of each tick.
    //This also resets the clock or the clock would attempt to catch up on the previous ticks.
    //This is truncated to the closest exact nanosecond duration.
    pub fn set_ups(&mut self, ups: u8) -> () {
        self.reset();
        self.aim = ((1.0 / ups as f64) * 1e9) as u128;
    }

    pub fn get_ups(&self) -> u8 {
        (self.aim / 1e9 as u128) as u8
    }

    fn reset(&mut self) -> () {
        self.average = 0;
        self.update_count = 0;
        self.last_time = None;
    }

    pub fn tick(&mut self) -> Duration {
        self.update_count += 1;
        self.average = match self.last_time {
            Some(instant) => {
                ((self.average * self.update_count + instant.elapsed().as_nanos()) - self.average)
                    / self.update_count
            }
            None => self.aim,
        };
        self.last_time = Some(Instant::now());
        //Ideally this convertion shouldn't be needed and but one type could be used.
        let d = Duration::from_nanos((2 * self.aim - self.average).try_into().expect(
            "self.average is bigger than twice the aim causing an unexpected integer overflow!",
        ));
        d
    }

    pub fn pause(&mut self) -> () {
        self.last_time = None;
    }

    pub fn new(ups: u8) -> Clock {
        let mut c = Clock {
            aim: 0,
            average: 0,
            update_count: 0,
            last_time: None,
        };
        c.set_ups(ups);
        c
    }
}
