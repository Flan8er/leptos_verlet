use bevy::prelude::*;

use crate::{
    core::parameters::{HALF_CAMERA_HEIGHT, Point, Stick},
    interaction::window_bounds::SimulationBounds,
};

const POINT_SIZE: f32 = 0.025; // m (0.025m == 25mm)
const STICK_SIZE: f32 = 0.01; // m (0.025m == 25mm)
const GRID_GAP: f32 = 0.1; // m
const CONTROL_BAR_HEIGHT: f32 = 0.075;
const FLOOR_OFFSET: f32 = 0.25; // m

pub fn spawn_cloth(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    point_material: &Handle<StandardMaterial>,
    stick_material: &Handle<StandardMaterial>,
    bounds: &Res<SimulationBounds>,
) {
    let point_mesh = meshes.add(Sphere::default());
    let stick_mesh = meshes.add(Cuboid::default());

    // Get the bounds of the simulation
    let window_width = bounds.width;
    let window_height = *HALF_CAMERA_HEIGHT * 2.;

    // Define cloth bounding box
    let points_x_tot = (window_width / GRID_GAP).floor() as u32;
    let points_y_tot =
        ((window_height - FLOOR_OFFSET - CONTROL_BAR_HEIGHT) / GRID_GAP).floor() as u32;

    let initial_x =
        (window_width - (GRID_GAP * points_x_tot as f32)) / 2. - GRID_GAP - window_width / 2.;
    let mut previous_position = Vec2::new(
        initial_x,
        window_height - (window_height - (GRID_GAP * points_y_tot as f32)) + FLOOR_OFFSET
            - CONTROL_BAR_HEIGHT,
    );

    // Keep an index of previously spawned points
    let mut ids: Vec<Vec<(Entity, Point)>> = vec![vec![]];

    for y_index in 0..points_y_tot {
        for x_index in 0..points_x_tot {
            let mut point_pos = previous_position;
            point_pos[0] += GRID_GAP;
            previous_position = point_pos;

            let new_point = Point::new(point_pos.extend(0.), point_pos.extend(0.), false);
            let new_point_id = commands
                .spawn((
                    Mesh3d(point_mesh.clone()),
                    MeshMaterial3d(point_material.clone()),
                    Transform::from_translation(new_point.position)
                        .with_scale(Vec3::splat(POINT_SIZE)),
                    new_point,
                ))
                .id();

            // Add the point to the list of created points
            ids[y_index as usize].push((new_point_id, new_point));

            // If the index isnt the first index - connect the previous with a stick
            if x_index != 0 {
                let prev_point = ids[y_index as usize][x_index as usize - 1].clone();

                // Create a stick joining the two points
                // Create a 3D position of the points - say they're positioned along the X-Y plane
                let spacial_point_1 = prev_point.1.position;
                let spacial_point_2 = new_point.position;

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
                        scale: Vec3::new(
                            spacial_point_2.distance(spacial_point_1),
                            STICK_SIZE,
                            STICK_SIZE,
                        ),
                    },
                    Stick::new(
                        prev_point.0,
                        new_point_id,
                        spacial_point_2.distance(spacial_point_1),
                    ),
                ));
            }
        }

        // If the index isnt the first index - connect the previous with a stick
        if y_index != 0 {
            for x_index in 0..points_x_tot {
                // Connect the top point to the bottom point
                let top_point = ids[y_index as usize - 1][x_index as usize].clone();
                let bottom_point = ids[y_index as usize][x_index as usize].clone();

                // Create a stick joining the two points
                // Create a 3D position of the points - say they're positioned along the X-Y plane
                let spacial_point_1 = top_point.1.position;
                let spacial_point_2 = bottom_point.1.position;

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
                        scale: Vec3::new(
                            spacial_point_2.distance(spacial_point_1),
                            STICK_SIZE,
                            STICK_SIZE,
                        ),
                    },
                    Stick::new(
                        top_point.0,
                        bottom_point.0,
                        spacial_point_2.distance(spacial_point_1),
                    ),
                ));
            }
        }

        if y_index != points_y_tot - 1 {
            // Create a new working vector for the next iteration
            ids.push(vec![]);

            // Reset the x position to it's default
            previous_position[0] = initial_x;

            // Increment the y position to it's next value
            previous_position[1] -= GRID_GAP;
        }
    }
}
