#![no_std]

pub trait Callback{
    fn call(&mut self);
}