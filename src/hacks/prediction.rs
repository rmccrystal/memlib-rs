use crate::math::{Vector3, Angles2};
use std::time::Instant;
use linreg::{linear_regression, linear_regression_of};
use std::collections::VecDeque;

pub struct Target {
    pub position: Vector3,
    pub velocity: Vector3,
    pub acceleration: Vector3,
}

impl Target {
    // TODO: Implement acceleration
    pub fn from_location_history(position: &Vector3, history: &VecDeque<(Instant, Vector3)>) -> Self {
        // Normalize history to start at 0 seconds
        let start_instant = history[0].0;
        let history: Vec<_> = history.iter()
            .map(|i| ((i.0 - start_instant).as_secs_f32(), i.1))
            .collect();

        let history_x: Vec<_> = history.iter().map(|n| (n.0, n.1.x)).collect();
        let history_y: Vec<_> = history.iter().map(|n| (n.0, n.1.y)).collect();
        let history_z: Vec<_> = history.iter().map(|n| (n.0, n.1.z)).collect();

        let (vel_x, _) = linear_regression_of(&history_x).expect("Error running linear regression on target's position history");
        let (vel_y, _) = linear_regression_of(&history_y).expect("Error running linear regression on target's position history");
        let (vel_z, _) = linear_regression_of(&history_z).expect("Error running linear regression on target's position history");

        Self {
            position: position.clone(),
            velocity: Vector3 { x: vel_x, y: vel_y, z: vel_z },
            acceleration: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
        }
    }

    pub fn get_location_at(&self, time: f32) -> Vector3 {
        self.position + (self.velocity * time) + (self.acceleration * (0.5 * time * time))
    }
}

pub struct Projectile {
    pub source_pos: Vector3,
    pub velocity: f32,
    // gravity in units/sec^2
    pub gravity: f32,
}

impl Projectile {
    pub fn time_to_reach(&self, target: &Vector3) -> f32 {
        let dis = (target - self.source_pos).length();
        // d=r*t; t=d/r
        dis / self.velocity
    }

    /// Returns the projectile's total y drop when reaching a target
    pub fn drop_to_target(&self, target: &Vector3) -> f32 {
        let time_to_reach = self.time_to_reach(&target);
        // 1/2 at^2
        0.5 * self.gravity * time_to_reach.powi(2)
    }
}

/// Runs a prediction algorithm on a target given a projectile. Returns the position to aim at
/// based on the time it takes for the projectile to hit the target
pub fn run_prediction(target: &Target, projectile: &Projectile) -> Vector3 {
    let time_to_reach = projectile.time_to_reach(&target.position);
    target.get_location_at(time_to_reach)
}

/// Modifies a target_pos to account for bullet drop according to the projectile
pub fn run_bullet_drop(target_pos: &Vector3, projectile: &Projectile) -> Vector3 {
    let total_drop = projectile.drop_to_target(&target_pos);
    target_pos + Vector3 { x: 0.0, y: 0.0, z: total_drop }
}
