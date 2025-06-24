use bevy::prelude::*;

use crate::{
    core::parameters::{HALF_CAMERA_HEIGHT, MODIFICATION_RADIUS, Point, Stick},
    interaction::{state::SimulationPlayState, window_bounds::SimulationBounds},
    objects::{cloth::spawn_cloth, cube::spawn_cube, rope::spawn_rope, square::spawn_square},
};

#[derive(Event, Clone, PartialEq)]
pub enum ModifyEventType {
    Left(RelativeWindowPosition),
    Right(RelativeWindowPosition),
    Middle(RelativeWindowPosition),
    Move(RelativeWindowPosition),
    Release(RelativeWindowPosition),
}
#[derive(Clone, PartialEq)]
pub struct RelativeWindowPosition {
    pub event_x: f32,
    pub event_y: f32,
    pub container_h: f32,
    pub container_w: f32,
}
impl RelativeWindowPosition {
    /// Takes in some incoming event location (in pixels) and outputs the ray
    /// that coincides with that point in Bevy unit space.
    pub fn create_ray(&self, camera: &Camera, camera_transform: &GlobalTransform) -> Ray3d {
        if let Ok(cast_ray) =
            camera.viewport_to_world(camera_transform, Vec2::new(self.event_x, self.event_y))
        {
            cast_ray
        } else {
            Ray3d::new(Vec3::new(0., 0., 0.), Dir3::NEG_Z)
        }
    }
}

#[derive(Event, Clone, PartialEq, States, Debug, Hash, Eq, Default)]
pub enum ModificationTarget {
    /// Left click to drop a point.
    Point,
    /// Click on two points two create a stick between them.
    Line,
    /// Lock/fix a point in place.
    Lock,
    /// Cut is tracking the "desire" to eventually cut.
    Cut,
    /// Cutting is tracking to "activity" of cutting.
    /// Cutting requires the left-click be held down and dragged over a stick.
    Cutting,
    /// Spawn a rope at a selected location.
    SpawnRope,
    /// Spawn a square at a selected location.
    SpawnSquare,
    /// Spawn a clock filling available sceen space.
    SpawnCloth,
    /// Spawn a cube at a selected location.
    SpawnCube,
    /// Right click on a point to delete it.
    Delete,
    PointInfo,
    #[default]
    None,
}

#[derive(Component)]
pub struct LineConnections {
    pub p0: Option<Entity>,
    pub p1: Option<Entity>,
}

pub fn handle_target_change(
    mut next_state: ResMut<NextState<SimulationPlayState>>,
    mut modification_target: ResMut<NextState<ModificationTarget>>,
    mut event_reader: EventReader<ModificationTarget>,
    mut commands: Commands,
    line_query: Query<(Entity, &mut LineConnections)>,
) {
    for event in event_reader.read() {
        // Pause the current simulation to open the gate for the editor logic
        // Only pause if an actual selection was made
        if event != &ModificationTarget::None {
            next_state.set(SimulationPlayState::Paused);
        }
        if event == &ModificationTarget::Line {
            purge_line(&line_query, &mut commands);
        }
        modification_target.set(event.clone())
    }
}

pub fn handle_modification_event(
    mut commands: Commands,
    mut event_reader: EventReader<ModifyEventType>,
    current_target: Res<State<ModificationTarget>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut stick_query: Query<(Entity, &mut Stick)>,
    mut params: ParamSet<(
        Query<&Point>,
        Query<(&mut MeshMaterial3d<StandardMaterial>, &mut Point)>,
        Query<(Entity, &Point)>,
    )>,
    mut next_target: ResMut<NextState<ModificationTarget>>,
    mut line_query: Query<(Entity, &mut LineConnections)>,
    bounds: Res<SimulationBounds>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    let point_material = materials.add(StandardMaterial::from(Color::WHITE));
    let stick_material = materials.add(StandardMaterial::from(Color::srgba(1., 1., 1., 0.5)));

    for event in event_reader.read() {
        match event {
            ModifyEventType::Left(relative_pos) => {
                let (camera, camera_transform) = match camera.get_single() {
                    Ok(queried_entity) => queried_entity,
                    Err(_) => continue,
                };

                let ray = relative_pos.create_ray(camera, camera_transform);

                let view_plane_world_pos = match ray_coords_at(ray, 0.) {
                    Some(coordinates) => coordinates,
                    None => continue,
                };

                match current_target.get() {
                    ModificationTarget::Point => {
                        let point = Point::new(view_plane_world_pos, view_plane_world_pos, false);
                        point.spawn(&mut commands, &mut meshes, &point_material);
                    }
                    ModificationTarget::Line => spawn_stick(
                        ray,
                        &mut line_query,
                        &params.p2(),
                        &mut commands,
                        &mut meshes,
                        &stick_material,
                    ),
                    ModificationTarget::Lock => {
                        lock_affected_points(ray, &mut params.p1(), &mut materials);
                    }
                    ModificationTarget::Cut => next_target.set(ModificationTarget::Cutting),
                    ModificationTarget::SpawnSquare => {
                        spawn_square(
                            &mut commands,
                            &mut meshes,
                            point_material.clone(),
                            stick_material.clone(),
                            view_plane_world_pos,
                        );
                    }
                    ModificationTarget::SpawnRope => {
                        let point_material = materials.add(StandardMaterial::from(Color::WHITE));
                        let stick_material =
                            materials.add(StandardMaterial::from(Color::srgba(1., 1., 1., 0.5)));
                        spawn_rope(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            point_material,
                            stick_material,
                            &bounds,
                            view_plane_world_pos,
                        );
                    }
                    ModificationTarget::SpawnCloth => spawn_cloth(
                        &mut commands,
                        &mut meshes,
                        &point_material,
                        &stick_material,
                        &bounds,
                    ),
                    ModificationTarget::SpawnCube => spawn_cube(
                        &mut commands,
                        &mut meshes,
                        &point_material,
                        &stick_material,
                        &view_plane_world_pos,
                    ),
                    _ => (),
                }
            }
            ModifyEventType::Right(relative_pos) => {}
            ModifyEventType::Middle(relative_pos) => {}
            ModifyEventType::Move(relative_pos) => {
                let (camera, camera_transform) = match camera.get_single() {
                    Ok(queried_entity) => queried_entity,
                    Err(_) => continue,
                };

                let ray = relative_pos.create_ray(camera, camera_transform);

                match current_target.get() {
                    ModificationTarget::Cutting => {
                        cut_sticks(ray, &mut stick_query, &params.p0(), &mut commands)
                    }
                    _ => (),
                }
            }
            ModifyEventType::Release(_) => {
                match current_target.get() {
                    ModificationTarget::Cutting => {
                        // When mouse is released, stop tracking cuts
                        next_target.set(ModificationTarget::Cut)
                    }
                    _ => (),
                }
            }
        }
    }
}

/// Purge any LineConnection by clearing out old LineConnections and creating a clean one
fn purge_line(line_query: &Query<(Entity, &mut LineConnections)>, commands: &mut Commands) {
    // Remove all remaining line components when a target changes - this is just a sanitation step
    for (entity, _) in line_query.iter() {
        commands.entity(entity).despawn();
    }

    // Spawn a line component to be used imediately upon selecting "Line"
    commands.spawn(LineConnections { p0: None, p1: None });
}

fn lock_affected_points(
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
    material: &Handle<StandardMaterial>,
) {
    // Ideally there is only one line at any point
    let (_, mut line) = match line_query.get_single_mut() {
        Ok(query_item) => query_item,
        Err(_) => {
            return;
        }
    };

    let stick_mesh = meshes.add(Cuboid::default());

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
                        let spacial_point_1 = p1.1.position; //.extend(0.)
                        let spacial_point_2 = p2.1.position; //.extend(0.)

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

fn cut_sticks(
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

fn sample_points_along_line(start: Vec3, end: Vec3, spacing: f32) -> Vec<Vec3> {
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

fn ray_coords_at(ray: Ray3d, target_z: f32) -> Option<Vec3> {
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
fn point_on_ray(ray: &Ray3d, point: Vec3, tolerance: f32) -> bool {
    let origin_to_point = point - ray.origin;
    let direction: Vec3 = ray.direction.into();

    let dot = origin_to_point.dot(direction);
    if dot < 0.0 {
        return false; // Point is behind the ray
    }

    let projected = ray.origin + direction * dot;

    (projected - point).length_squared() <= tolerance * tolerance
}
