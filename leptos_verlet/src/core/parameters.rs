use bevy::prelude::*;
use once_cell::sync::Lazy;

use crate::{
    core::spawner::{SpawnNode, spawner},
    prelude::{MaterialType, MeshType},
};

// Setting the camera some distance away while keeping the "floor" at y=0
pub const CAMERA_FOV: f32 = std::f32::consts::PI / 4.;
pub const CAMERA_DISTANCE: f32 = 4.; // m
pub static HALF_CAMERA_HEIGHT: Lazy<f32> = Lazy::new(|| CAMERA_DISTANCE * (CAMERA_FOV / 2.0).tan());

/// Percent of energy KEPT after each contact with a collision surface.
pub const COEFF_RESTITUTION: f32 = 0.95;
/// Percent of energy KEPT after each contact with the floor.
pub const FRICTION_RESTITUTION: f32 = 0.95;
/// m/s^2
pub const GRAVITY: f32 = 9.8;
/// Amount of energy KEPT each frame passing through the air.
/// Since this is energy reduced EVERY frame, only an
/// extremely small loss is needed.
pub const AIR_RESISTANCE: f32 = 0.995;
/// The value indicating the minimum distance change to update the rendering of a given frame.
/// Any distance change, per frame, below this value will not be reflected in the rendering.
pub const MIN_RENDER_DELTA: f32 = 0.001;
/// The distance from the mouse any modification request will disperse to nearby components.
pub const MODIFICATION_RADIUS: f32 = 0.03;

/// Central definition of each default geometry properties
pub const POINT_SIZE: f32 = 0.025; // m (0.025m == 25mm)
pub const STICK_SIZE: f32 = 0.01; // m (0.025m == 25mm)

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub position: Vec3,
    pub prev_position: Vec3,
    pub locked: bool,
}
impl Point {
    pub fn new(position: Vec3, prev_position: Vec3, locked: bool) -> Self {
        Self {
            position,
            prev_position,
            locked,
        }
    }

    pub fn update_properties(&mut self, time: &Res<Time>) {
        let velocity = self.calculate_affected_velocity();

        // new position = pos + vel + ½·a·dt²
        let dt = time.delta_secs();
        let acc = Vec3::new(0.0, -GRAVITY, 0.0);
        let new_pos = self.position + velocity + acc * dt * (1. / 120.);

        // shift “current” into “previous” for the next frame
        self.prev_position = self.position;
        self.position = new_pos;
    }

    /// Calculates the velocity based only on the points current vs previous position.
    pub fn calculate_velocity(self) -> Vec3 {
        self.position - self.prev_position
    }

    /// Calculates the velocity according to outside factors such as air
    /// resistance and friction.
    pub fn calculate_affected_velocity(self) -> Vec3 {
        let mut new_velocity = (self.position - self.prev_position) * AIR_RESISTANCE;

        if self.position[1] <= 0.001 {
            // Calculate the change in velocity due to friction losses.
            let current_velocity = self.calculate_velocity();
            new_velocity = current_velocity * FRICTION_RESTITUTION;
        }

        new_velocity
    }

    pub fn spawn(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        material: &mut ResMut<Assets<StandardMaterial>>,
        point_mesh: MeshType,
        point_material: MaterialType,
    ) {
        let point = SpawnNode {
            point: self.clone(),
            point_material,
            point_mesh,
            ..default()
        };
        let mesh_network = vec![point];

        spawner(mesh_network, commands, meshes, material);
    }
}

#[derive(Component)]
pub struct Stick {
    pub point1: Entity,
    pub point2: Entity,
    pub length: f32,
}
impl Stick {
    pub fn new(point1: Entity, point2: Entity, length: f32) -> Self {
        Self {
            point1,
            point2,
            length,
        }
    }
}
