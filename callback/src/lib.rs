#![no_std]

pub trait Callback{
    // Return false when callback aborted, return true else.
    fn call(&mut self) -> bool;
}

pub trait Stopper{
    fn should_stop(&mut self) -> bool;
}