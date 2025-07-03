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

#[derive(Resource, Copy, Clone, Debug)]
pub struct SimulationSettings {
    /// The number of iterations to converge the sticks to their proper positions.
    /// Higher values will result in less elasticity in the simulation bodies.
    pub converge_iterations: u16,
    /// A value to optimize rendering. Indicates the minimum distace the simulation points
    /// must travel in order to update the rendering. If any simulation point travels this distance
    /// the entire simulation will be rerendered.
    ///
    /// This value can be used if small perturbations can be be observed to keep them from being rendered.
    pub min_render_delta: f32,
    /// The value indicating the maximum number of unchanged frames that will trigger a rerender.
    /// If the min_render_delta hasn't been hit in this number of frames, the rendering will update.
    pub max_unchanged_frames: u32,
    /// Defines the point size of the builtin geometries (cloth, cube, rope, ...)
    ///
    /// Units are meters
    pub default_geometry_point_size: f32,
    /// Defines the stick size of the builtin geometries (cloth, cube, rope, ...)
    ///
    /// Units are meters
    pub default_geometry_stick_size: f32,
    /// The distance from the mouse any modification request will disperse to nearby simulation bodies.
    pub interaction_radius: f32,
    /// Percent of energy kept after each contact with a collision surface.
    pub coeff_restitution: f32,
    /// Percent of energy kept after each contact with the floor.
    /// A rolling/sliding (rolling isn't really simulated) object is in constant contact with the floor
    /// so this value is appied at every every frame - a little goes a long way.
    pub friction_restituation: f32,
    /// m/s^2
    pub gravity: f32,
    /// The amount of energy kept each frame a simulation point passes through the air.
    /// As this loss is applied every frame, a little goes a long way.
    pub air_resistance: f32,
    pub simulation_bounds: SimulationBounds,
    /// A value from 0-1 that is applied to filter out harsh velocity changes
    /// and can also be used to smooth out any "jitters" in a simulation body.
    ///
    /// A value of 0 will have no filtering and a value of 1 will halt any velocity changes.
    /// Higher values will seem to slow the simulation down and might need to be compensated with higher accelerations
    /// to keep an appearance of accurate-to-scale physics.
    ///
    /// How aggressively to shave off the discrete acceleration spikes.
    pub jerk_damping: f32,
    pub camera_fov: f32,
    pub camera_position: Vec3,
    pub camera_orientation: Quat,
    pub light_position: Vec3,
    pub light_orientation: Quat,
    pub light_luminosity: f32,
    pub ambient_light: f32,
}
impl Default for SimulationSettings {
    fn default() -> Self {
        let camera_fov = CAMERA_FOV;
        let camera_distance = CAMERA_DISTANCE;
        let half_camera_height = *HALF_CAMERA_HEIGHT;
        let camera_position = Vec3::new(0.0, half_camera_height, camera_distance);
        let light_position = Vec3::new(10., 10., 10.);

        Self {
            converge_iterations: 10,
            min_render_delta: 0.0,
            max_unchanged_frames: 120,
            default_geometry_point_size: 0.025,
            default_geometry_stick_size: 0.01,
            interaction_radius: 0.03,
            coeff_restitution: 0.95,
            friction_restituation: 0.95,
            gravity: 9.8,
            air_resistance: 0.995,
            simulation_bounds: SimulationBounds::new(true, true, true),
            jerk_damping: 0.4,
            camera_fov,
            camera_position,
            camera_orientation: Quat::IDENTITY,
            light_position,
            light_orientation: Quat::from_rotation_arc(
                -Vec3::Z,
                (Vec3::ZERO - light_position).normalize(),
            ),
            light_luminosity: 10_000.0,
            ambient_light: 80.,
        }
    }
}

#[derive(Copy, Clone, Debug)]
/// The bound value is calculated as an event based on the container size.
/// The y-bounds has the floor set to y=0.
pub struct SimulationBounds {
    pub x: (bool, f32),
    pub y: (bool, f32),
    pub z: (bool, f32),
}
impl SimulationBounds {
    pub fn new(x: bool, y: bool, z: bool) -> Self {
        Self {
            x: (x, *HALF_CAMERA_HEIGHT * 2.),
            y: (y, *HALF_CAMERA_HEIGHT * 2.),
            z: (z, CAMERA_DISTANCE),
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct Point {
    /// The positon the point will be at NEXT
    pub position: Vec3,
    /// The current mathematical position the point is at
    pub prev_position: Vec3,
    /// Is the point locked in space
    pub locked: bool,
    /// The currect position the point is rendered at
    pub rendered_position: Vec3,
    /// The position the point was at on the previous frame
    pub previously_rendered_position: Vec3,
    /// Any fixed external forces that should act on the point (magnetic field, applied forces, ...).
    /// The forces act relative to the world coordinate system.
    ///
    /// Important: Gravity is applied at a simulation level. Gravity can be turned off if desired and
    /// only added to specific points.
    pub external_forces: Vec3,
}
impl Point {
    pub fn new(position: Vec3, prev_position: Vec3, locked: bool) -> Self {
        Self {
            position,
            prev_position,
            locked,
            rendered_position: position,
            previously_rendered_position: position,
            external_forces: Vec3::ZERO,
        }
    }
    pub fn new_with_options(
        position: Vec3,
        prev_position: Vec3,
        locked: bool,
        external_forces: Vec3,
    ) -> Self {
        Self {
            position,
            prev_position,
            locked,
            rendered_position: position,
            previously_rendered_position: position,
            external_forces,
        }
    }

    pub fn update_properties(&mut self, time: &Res<Time>, sim_settings: &SimulationSettings) {
        let velocity = self.calculate_affected_velocity(sim_settings);

        // new position = pos + vel + ½·a·dt²
        let dt = time.delta_secs();
        let acc = Vec3::new(0.0, -sim_settings.gravity, 0.0) + self.external_forces;
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
    pub fn calculate_affected_velocity(self, sim_settings: &SimulationSettings) -> Vec3 {
        let mut new_velocity = (self.position - self.prev_position) * sim_settings.air_resistance;

        if self.position[1] <= 0.001 {
            // Calculate the change in velocity due to friction losses.
            let current_velocity = self.calculate_velocity();
            new_velocity = current_velocity * sim_settings.friction_restituation;
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
