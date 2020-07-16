// Exports interfaces that structs can implement to use game agnostic cheat features

use crate::math;

/// An interface representing a player
pub trait Player {
    fn get_origin(&self) -> math::Vector3;
    fn get_name(&self) -> String;
    fn get_view_angles(&self) -> math::Rotation3;
    fn get_health(&self) -> i32;
    fn get_team_num(&self) -> i32;
    fn is_alive(&self) -> bool;
}
