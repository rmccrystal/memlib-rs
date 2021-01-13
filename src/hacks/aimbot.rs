use crate::math::{radians_to_deg, Angles2, Vector3};

/// Game agnostic implementation for an aimbot
/// Takes the source position, the enemy position, your view angles, and the smooth amount.
/// Returns the new view angles based on the input
pub fn calculate_aimbot(
    source_position: Vector3,
    target_position: Vector3,
    _view_angles: Angles2,
    _smooth: f32,
) -> Angles2 {
    let delta = target_position - source_position;
    let delta_length = delta.length();

    let pitch = radians_to_deg(-f32::asin(delta.z / delta_length));
    let yaw = radians_to_deg(f32::atan2(delta.y, delta.x));

    // TODO: Implement smoothing

    let mut new_angle = Angles2 { pitch, yaw };
    new_angle.clamp();

    new_angle
}
