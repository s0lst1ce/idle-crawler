use std::convert::TryInto;
use std::time::{Duration, Instant};

/// Enables tick duration to be consistant over time.
///
/// The Clock is used to determine how much time the game should sleep in-between each update.
/// To know how long the game should wait before the next update the `tick` method is used.
/// The Clock does not garantuee that all ticks will be as long, or that any tick will be the exact length.
/// However the average time spent sleeping will be exact.
///
/// The Clock can't make the game faster!
/// If the average time a tick should last (determined by the UPS) is lower than the time the game takes to update,
/// then the game state will be behind expectations.
/// However if this is only momentary, the clock will do its best to catch up. To prevent this behavior you should call `set_ups` again.
///
///Sometimes it is useful to suspend the clock for some time, whether it is because the game is paused
///or else. This is done through the `pause` method. To resume the clock simply `tick` again.
///
/// # Examples
///
/// Using the clock to make sure each tick takes in average 1/30s.
///```
///# use std::thread;
///# use server::clock::Clock;
///# let mut counter = 2;
///let mut clock = Clock::new(30); //we set clock to run at 30 Updates Per Second (UPS)
///loop {
///# counter -= 1;
/// //update the game
/// thread::sleep(clock.tick());
///# if counter == 0 {break;}
///}
///```
#[derive(Debug)]
pub struct Clock {
    aim: u128,
    average: u128,
    update_count: u128,
    last_time: Option<Instant>,
}

impl Clock {
    /// Sets the Clock UPS. This is also ran when making a new Clock so you only need to run when the requirements change during execution.
    /// This does not try to catch up, it will act as if a new Clock was just made.
    /// UPS are Updates Per Second, so if you want 30 of them the average time of a tick will be 0.03s.
    ///
    /// # Example
    ///
    /// ```
    ///# use server::clock::Clock;
    ///# use std::time::Duration;
    ///# let mut clock = Clock::new(1);
    /// clock.set_ups(10);
    /// assert_eq!(clock.get_ups(), 10);
    /// assert_eq!(clock.tick(), Duration::from_millis(100));
    ///```
    //This also resets the clock or the clock would attempt to catch up on the previous ticks.
    //This is truncated to the closest exact nanosecond duration.
    pub fn set_ups(&mut self, ups: u8) -> () {
        self.reset();
        self.aim = ((1.0 / ups as f64) * 1e9) as u128;
    }

    /// Returns the current UPS tracked by the Clock.
    pub fn get_ups(&self) -> u8 {
        (1e9 as u128 / self.aim) as u8
    }

    fn reset(&mut self) -> () {
        self.average = 0;
        self.update_count = 0;
        self.last_time = None;
    }

    /// Gives the Duration for which the game should wait before the next update.
    /// This takes the average update time into account and asserts all ticks are the same length in average.
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

    /// Pauses the Clock. Use this if you intend to stop sleeping between ticks or if you suspend the game.
    /// To resume the Clock simply call `tick` again. A resumed clock will not attempt to catch up ticks lost during the pause.
    ///
    /// # Example
    /// We pause the clock. Note that the first `tick` call resuming the clock will always be exactly equal to the targeted tick time.
    /// ```
    ///# use std::thread;
    ///# use server::clock::Clock;
    ///# use std::time::Duration;
    ///# let mut clock = Clock::new(10);
    /// clock.pause();
    /// thread::sleep(Duration::from_millis(200)); //the value here is arbitrary
    /// assert_eq!(clock.tick(), Duration::from_millis(100));
    ///```
    pub fn pause(&mut self) -> () {
        self.last_time = None;
    }

    /// Creates a new clock with the Updates Per Second set to `ups`. To change UPS prefer using `set_ups` to creating a new Clock.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_ups() {
        let clock = Clock::new(10);
        assert_eq!(clock.aim, 1e8 as u128);
    }

    #[test]
    fn test_get_ups() {
        let clock = Clock::new(10);
        assert_eq!(clock.get_ups(), 10);
    }

    //I would love to check more of the logic. Unfortunately I don't know how to make up for the random execution time :(
    #[test]
    fn test_tick() {
        let mut clock = Clock::new(10);
        assert_eq!(clock.tick().as_nanos(), clock.aim);
    }
}
