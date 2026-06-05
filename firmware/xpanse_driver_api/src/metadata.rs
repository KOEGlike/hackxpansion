//! Metadata for apps about the drivers, like what physiscal slot the module is in, that the driver uses,
//! and what type of module is used by the driver

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slots {
    FrontLeft,
    FrontRight,
    BackLeft,
    BackRight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Module {
    TestModule,
}
