use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{
    core::parameters::{Point, Stick},
    interaction::window_bounds::SimulationBounds,
};

const POINT_SIZE: f32 = 0.025; // m (0.025m == 25mm)
const STICK_SIZE: f32 = 0.01; // m (0.025m == 25mm)
/// Smaller stick length will result in a denser rope
const STICK_LENGTH: f32 = 0.025; // m
const DROP_ANGLE: f32 = 35.; // deg
const ROPE_LENGTH: f32 = 1.5; // m

pub fn spawn_rope(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    point_material: Handle<StandardMaterial>,
    stick_material: Handle<StandardMaterial>,
    bounds: &Res<SimulationBounds>,
    position: Vec3,
) {
    let point_mesh = meshes.add(Sphere::default());
    let stick_mesh = meshes.add(Cuboid::default());

    let locked_color = materials.add(StandardMaterial::from(Color::srgb(1., 0., 0.)));

    // Create an initial point locked at some starting position.
    let mut parent_point = Point::new(position, position, true);
    let mut parent_point_id = commands
        .spawn((
            Mesh3d(point_mesh.clone()),
            MeshMaterial3d(locked_color),
            Transform::from_translation(parent_point.position).with_scale(Vec3::splat(POINT_SIZE)),
            parent_point,
        ))
        .id();

    // Total number to links should be just enough to **kiss** the floor
    let sticks_tot = (ROPE_LENGTH / STICK_LENGTH).floor() as u32;

    // Loop through the desired number of sticks
    // For the requested number of sticks create a point and connect it to the previous parent point
    for _ in 0..sticks_tot {
        // Convert the desired angle into rad
        let theta = DROP_ANGLE * PI / 180.;

        // Calculate change in x position in the -X direction
        let mut point_position =
            Vec3::new(-STICK_LENGTH * theta.sin(), STICK_LENGTH * theta.cos(), 0.)
                + parent_point.position;
        // Constrain point to be within boundary
        if point_position[0] <= -bounds.width / 2. {
            let constrained_x = -bounds.width / 2. + 0.001;
            let diff = constrained_x - point_position[0];

            point_position = Vec3::new(constrained_x, point_position[1] + diff, 0.);
        } else if point_position[0] >= bounds.width / 2. {
            let constrained_x = bounds.width / 2. - 0.001;
            let diff = point_position[0] - constrained_x;

            point_position = Vec3::new(constrained_x, point_position[1] + diff, 0.);
        }

        // Create and spawn the new point
        let child_point = Point::new(point_position, point_position, false);
        let child_point_id = commands
            .spawn((
                Mesh3d(point_mesh.clone()),
                MeshMaterial3d(point_material.clone()),
                Transform::from_translation(child_point.position)
                    .with_scale(Vec3::splat(POINT_SIZE)),
                child_point,
            ))
            .id();

        // Create a stick joining the parent to the child
        // Create a 3D position of the points - say they're positioned along the X-Y plane
        let spacial_point_1 = parent_point.position;
        let spacial_point_2 = child_point.position;

        // Determine the objects rotation quaternion
        let diff = spacial_point_2 - spacial_point_1;
        let rot = Quat::from_rotation_arc(Vec3::X, diff.normalize());

        // Spawn the stick linking the two points
        commands.spawn((
            Mesh3d(stick_mesh.clone()),
            MeshMaterial3d(stick_material.clone()),
            Transform {
                translation: (spacial_point_1 + spacial_point_2) * 0.5,
                rotation: rot,
                scale: Vec3::new(STICK_LENGTH, STICK_SIZE, STICK_SIZE),
            },
            Stick::new(parent_point_id, child_point_id, STICK_LENGTH),
        ));

        // For the next iteration update the parent to be the current child
        parent_point = child_point;
        parent_point_id = child_point_id;
    }
}
