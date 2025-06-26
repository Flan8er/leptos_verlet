use bevy::prelude::*;

use crate::{
    core::{
        parameters::{MODIFICATION_RADIUS, Stick},
        spawner::material_from_descriptor,
    },
    plugins::{info::plugin::ActiveInfoTarget, modification::plugin::LineConnections},
    prelude::{MaterialType, Point},
};

pub fn perge_info_target(
    point_query: &Query<(Entity, &Point), With<ActiveInfoTarget>>,
    commands: &mut Commands,
) {
    for (entity, _) in point_query.iter() {
        commands.entity(entity).remove::<ActiveInfoTarget>();
    }
}

pub fn point_info(cast_ray: Ray3d, point_query: &Query<(Entity, &Point)>, commands: &mut Commands) {
    for (entity, point) in point_query.iter() {
        if point_on_ray(&cast_ray, point.position, MODIFICATION_RADIUS) {
            commands.entity(entity).insert(ActiveInfoTarget);
        }
    }
}

/// Purge any LineConnection by clearing out old LineConnections and creating a clean one
pub fn purge_line(line_query: &Query<(Entity, &mut LineConnections)>, commands: &mut Commands) {
    // Remove all remaining line components when a target changes - this is just a sanitation step
    for (entity, _) in line_query.iter() {
        commands.entity(entity).despawn();
    }

    // Spawn a line component to be used imediately upon selecting "Line"
    commands.spawn(LineConnections { p0: None, p1: None });
}

pub fn lock_affected_points(
    cast_ray: Ray3d,
    points: &mut Query<(&mut MeshMaterial3d<StandardMaterial>, &mut Point)>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Loop through all the spawned points that should be analyzed for selection
    for (mut material, mut pt) in points {
        // Check to see if the point lies on the ray
        if point_on_ray(&cast_ray, pt.position, MODIFICATION_RADIUS) {
            pt.locked = !pt.locked;

            let color = if pt.locked {
                Color::srgb(1., 0., 0.)
            } else {
                Color::WHITE
            };

            let new_handle = materials.add(StandardMaterial::from(color));
            material.0 = new_handle;
        }
    }
}

pub fn spawn_stick(
    cast_ray: Ray3d,
    line_query: &mut Query<(Entity, &mut LineConnections)>,
    points: &Query<(Entity, &Point)>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    material: MaterialType,
) {
    // Ideally there is only one line at any point
    let (_, mut line) = match line_query.get_single_mut() {
        Ok(query_item) => query_item,
        Err(_) => {
            return;
        }
    };

    let stick_mesh = meshes.add(Cuboid::default());
    let material = material_from_descriptor(&material, materials);

    // Find, optionally, the point that is at the event coordinates
    for (entity, point) in points {
        if point_on_ray(&cast_ray, point.position, MODIFICATION_RADIUS) {
            // Attach this entity to one of the arms of the LineConntection
            match (line.p0, line.p1) {
                (None, None) => {
                    line.p0 = Some(entity);
                }
                (Some(_), None) => {
                    line.p1 = Some(entity);
                }
                (None, Some(_)) => {
                    line.p0 = Some(entity);
                }
                (Some(_), Some(_)) => {
                    panic!("Err: Line connection wasnt cleaned up.");
                }
            }

            match (line.p0, line.p1) {
                (Some(p0_id), Some(p1_id)) => {
                    // The line connection is complete and ready to be spawned

                    // Retreive the physical component from the IDs
                    if let Ok([p1, p2]) = points.get_many([p0_id, p1_id]) {
                        // Calculate stick position and orientation
                        let spacial_point_1 = p1.1.position;
                        let spacial_point_2 = p2.1.position;

                        let diff = spacial_point_2 - spacial_point_1;
                        let rot = Quat::from_rotation_arc(Vec3::X, diff.normalize());

                        // Spawn the stick connecting the two points
                        commands.spawn((
                            Mesh3d(stick_mesh.clone()),
                            MeshMaterial3d(material.clone()),
                            Transform {
                                translation: (spacial_point_1 + spacial_point_2) * 0.5,
                                rotation: rot,
                                scale: Vec3::new(diff.length(), 0.01, 0.01),
                            },
                            Stick::new(p0_id, p1_id, spacial_point_1.distance(spacial_point_2)),
                        ));

                        // After spawning the stick, remove the LineConntections points so a fresh line can be started
                        line.p0 = None;
                        line.p1 = None;
                    }
                }
                _ => (),
            }
        }
    }
}

pub fn cut_sticks(
    cast_ray: Ray3d,
    sticks: &mut Query<(Entity, &mut Stick)>,
    points: &Query<&Point>,
    commands: &mut Commands,
) {
    for (entity, stick) in sticks {
        if let Ok([p1, p2]) = points.get_many([stick.point1, stick.point2]) {
            // Create the list of points between this sticks endpoints
            let sample_points =
                sample_points_along_line(p1.position, p2.position, MODIFICATION_RADIUS);
            for point in sample_points {
                // Check to see if the point lies on the ray
                if point_on_ray(&cast_ray, point, MODIFICATION_RADIUS) {
                    // Remove this line from the simulation
                    commands.entity(entity).despawn();
                    break;
                }
            }
        }
    }
}

pub fn sample_points_along_line(start: Vec3, end: Vec3, spacing: f32) -> Vec<Vec3> {
    assert!(spacing > 0.0, "spacing must be positive");

    let delta = end - start;
    let total_length = delta.length();
    if total_length == 0.0 {
        // degenerate case: start == end
        return vec![start];
    }

    let direction = delta / total_length; // unit vector
    let mut points = Vec::new();

    points.push(start);

    // march along the line in steps of `spacing`
    let mut traveled = spacing;
    while traveled < total_length {
        points.push(start + direction * traveled);
        traveled += spacing;
    }

    points.push(end);
    points
}

pub fn ray_coords_at(ray: Ray3d, target_z: f32) -> Option<Vec3> {
    let origin = ray.origin;
    let direction = ray.direction;

    if direction.z.abs() < f32::EPSILON {
        // Ray is parallel to the z-plane; may never intersect
        return None;
    }

    let t = (target_z - origin.z) / direction.z;
    Some(origin + direction * t)
}

/// A check to see if any 3D point is places inline with a ray.
pub fn point_on_ray(ray: &Ray3d, point: Vec3, tolerance: f32) -> bool {
    let origin_to_point = point - ray.origin;
    let direction: Vec3 = ray.direction.into();

    // How much of vector A lies in some direction.
    let dot = origin_to_point.dot(direction);

    // Exclude points behind the ray’s origin.
    if dot < 0.0 {
        return false;
    }

    // Find the closest point on the ray to the given point
    let projected = ray.origin + direction * dot;

    // Check how far the point is from its closest point on the ray.
    (projected - point).length_squared() <= tolerance * tolerance
}
