// To avoid stack overflow
#![feature(box_syntax)]
#![feature(min_type_alias_impl_trait)]
#![feature(maybe_uninit_uninit_array)]

pub mod common;
pub mod components;
pub mod math;
pub mod renderer;
pub mod resources;
pub mod plugins;
pub mod world;
