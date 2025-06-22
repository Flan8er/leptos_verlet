use bevy::prelude::*;

use crate::core::parameters::{Point, Stick};

const POINT_SIZE: f32 = 0.025; // m (0.025m == 25mm)
const STICK_SIZE: f32 = 0.01; // m (0.025m == 25mm)

pub fn spawn_square(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    point_material: Handle<StandardMaterial>,
    stick_material: Handle<StandardMaterial>,
    position: Vec3,
) {
    let square_size = 0.45;

    let bottom_left = Point::new(
        Vec3::new(
            position[0] - square_size,
            position[1] - square_size,
            position[2],
        ),
        Vec3::new(
            position[0] - square_size,
            position[1] - square_size,
            position[2] + 0.1,
        ),
        false,
    );
    let bottom_right = Point::new(
        Vec3::new(
            position[0] + square_size,
            position[1] - square_size,
            position[2],
        ),
        Vec3::new(
            position[0] + square_size,
            position[1] - square_size,
            position[2] + 0.1,
        ),
        false,
    );
    let top_right = Point::new(
        Vec3::new(
            position[0] + square_size,
            position[1] + square_size,
            position[2],
        ),
        Vec3::new(
            position[0] + square_size,
            position[1] + square_size,
            position[2],
        ),
        false,
    );
    let top_left = Point::new(
        Vec3::new(
            position[0] - square_size,
            position[1] + square_size,
            position[2],
        ),
        Vec3::new(
            position[0] - square_size,
            position[1] + square_size,
            position[2],
        ),
        false,
    );

    let stick_mesh = meshes.add(Cuboid::default());

    // Spawn the points of the square
    let shape_points = vec![bottom_left, bottom_right, top_right, top_left];
    let spawned_ids = spawn_points(
        commands,
        meshes,
        point_material.clone(),
        shape_points.clone(),
    );

    // Connect the points with a stick
    // Perform one extra connection to rigidly link a set of corners
    for mut i in 0..shape_points.len() + 1 {
        let j = if i == shape_points.len() - 1 {
            // Connect this corner back to the first corner (i.e. complete the border of the square)
            0
        } else if i == shape_points.len() {
            // Connect some corner to the apposing corner
            // Set i back to zero to keep the indexing in bounds of the array
            i = 0;
            // Return an offset of two (apposing corner)
            i + 2
        } else {
            // Conntect this point to the next one in the array
            i + 1
        };

        // Create a 3D position of the point - say it's positioned along the X-Y plane
        let spacial_point_1 = shape_points[i].position; //.extend(0.)
        let point_1_id = spawned_ids[i];
        let spacial_point_2 = shape_points[j].position; //.extend(0.)
        let point_2_id = spawned_ids[j];

        // Calculate the distance between the two points
        let diff = spacial_point_2 - spacial_point_1;
        let distance = diff.length();

        // Determine the objects rotation quaternion
        let rot = Quat::from_rotation_arc(Vec3::X, diff.normalize());

        // Spawn the stick linking the two points
        commands.spawn((
            Mesh3d(stick_mesh.clone()),
            MeshMaterial3d(stick_material.clone()),
            Transform {
                translation: (spacial_point_1 + spacial_point_2) * 0.5,
                rotation: rot,
                scale: Vec3::new(distance, STICK_SIZE, STICK_SIZE),
            },
            Stick::new(point_1_id, point_2_id, distance),
        ));
    }
}

fn spawn_points(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<StandardMaterial>,
    points: Vec<Point>,
) -> Vec<Entity> {
    // Keep track of the points that are spawned
    let mut spawned_entities = Vec::new();

    // Create the point mesh
    let point_mesh = meshes.add(Sphere::default());

    // Spawn all the requested points
    for pt in points {
        // Spawn the point
        let point_id = commands
            .spawn((
                Mesh3d(point_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(pt.position) //.extend(0.)
                    .with_scale(Vec3::splat(POINT_SIZE)),
                pt,
            ))
            .id();

        // Add the entity to the lists of spawns
        spawned_entities.push(point_id);
    }

    spawned_entities
}
