use std::ops::{Add, Sub};
use std::f32::consts::PI;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0 && self.z == 0.0
    }

    pub fn length(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }
}

impl Add for Vector3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self{
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Vector3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self{
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}



#[repr(C)]
#[derive(Debug, Clone)]
/// Represent pitch / yaw view angles
/// Pitch: +down / -up
/// Yaw: +right / -left
pub struct Angles2 {
    pub pitch: f32,
    pub yaw: f32,
}

impl Angles2 {
    /// Creates a new Angles2 struct using a pitch and yaw and clamps it
    pub fn new(pitch: f32, yaw: f32) -> Self {
        let mut new_angles = Self{pitch, yaw};
        new_angles.clamp();
        new_angles
    }

    /// Clamps the angles between:
    /// Pitch: [-90, 90]
    /// Yaw: [-180, 180]
    pub fn clamp(&mut self) {
        while self.pitch > 90.0 {
            self.pitch -= 90.0
        }
        while self.pitch < -90.0 {
            self.pitch += 90.0
        }

        while self.yaw > 180.0 {
            self.yaw -= 180.0
        }
        while self.yaw < -180.0 {
            self.yaw += 180.0
        }
    }
}


pub fn deg_to_radians(degrees: f32) -> f32 {
    degrees * (PI / 180.0)
}
pub fn radians_to_deg(radians: f32) -> f32 {
    radians * (180.0 / PI)
}