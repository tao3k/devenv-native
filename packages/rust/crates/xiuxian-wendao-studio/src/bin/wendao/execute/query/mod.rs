//! Coordinates the Studio wendao execute query branch and keeps its child modules behind one documented reasoning-tree boundary.

mod command;
mod graphql;
mod rest;
mod sql;

pub(super) use command::handle;
