#![no_std]

pub trait Callback{
    fn call(&mut self);
}

pub trait Stopper{
    fn should_stop(&mut self) -> bool;
}