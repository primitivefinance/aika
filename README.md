# aika
[![Build](https://github.com/primitivefinance/aika/actions/workflows/build.yaml/badge.svg)](https://github.com/primitivefinance/aika/actions/workflows/build.yaml)
[![Rust](https://github.com/primitivefinance/aika/actions/workflows/rust.yaml/badge.svg)](https://github.com/primitivefinance/aika/actions/workflows/rust.yaml)

Discrete event simulation manager built in Rust 🦀 . Aika is designed to have a similar base syntax to [SimPy](https://gitlab.com/team-simpy/simpy), however, has the addition of a `Manager` to improve focus on large, repetitive, data intensive simulations.

This simulator utilizes `generators`, currently an expiremental feature of Rust nightly version 1.71.0. 

![](/assets/aika-clock.png)