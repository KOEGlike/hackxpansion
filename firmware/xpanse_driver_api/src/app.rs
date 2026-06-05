use alloc::boxed::Box;
use core::pin::Pin;
use core::future::Future;

pub trait App<R> {
    // Run the application
    fn run<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = ()> + 'a>>;
    
    // Check if dependencies are met
    fn can_run(registry: &R) -> bool where Self: Sized;

    // Instantiate the application, taking capabilities out of the registry
    fn new(registry: &mut R) -> Option<Self> where Self: Sized;
}
