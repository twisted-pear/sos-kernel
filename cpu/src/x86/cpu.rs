//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015-2016 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! Code for interacting with the `x86` CPU.
//!
//! Currently this module contains a quick implementation of CPU port
//! input and output, and little else.

#[path = "../x86_all/cpu/mod.rs"] mod cpu_all;
pub use self::cpu_all::*;

#[path = "../x86_all/interrupts/mod.rs"]
pub mod interrupts;
