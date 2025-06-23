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
    pub x: f32,
    pub y: f32,
    /// The ratio of container_width / container_height
    pub x_to_y: f32,
}
impl RelativeWindowPosition {
    pub fn to_bevy_coords(&self) -> Vec2 {
        Vec2::new(
            2. * *HALF_CAMERA_HEIGHT * self.x * self.x_to_y - *HALF_CAMERA_HEIGHT * self.x_to_y,
            (2. * *HALF_CAMERA_HEIGHT) * (1. - self.y),
        )
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
    camera: Query<&GlobalTransform, With<Camera3d>>,
) {
    let point_mesh = meshes.add(Sphere::default());
    let stick_mesh = meshes.add(Cuboid::default());
    let point_material = materials.add(StandardMaterial::from(Color::WHITE));
    let stick_material = materials.add(StandardMaterial::from(Color::srgba(1., 1., 1., 0.5)));

    for event in event_reader.read() {
        match event {
            ModifyEventType::Left(relative_pos) => {
                let event_coords = relative_pos.to_bevy_coords();

                match current_target.get() {
                    ModificationTarget::Point => {
                        spawn_point(event_coords, &mut commands, &point_mesh, &point_material);
                    }
                    ModificationTarget::Line => connect_points(
                        event_coords,
                        &mut line_query,
                        &params.p2(),
                        &mut commands,
                        &stick_mesh,
                        &stick_material,
                    ),
                    ModificationTarget::Lock => {
                        lock_affected_points(event_coords, &mut params.p1(), &mut materials);
                    }
                    ModificationTarget::Cut => next_target.set(ModificationTarget::Cutting),
                    ModificationTarget::SpawnSquare => {
                        spawn_square_at(event_coords, &mut commands, &mut meshes, &mut materials)
                    }
                    ModificationTarget::SpawnRope => spawn_rope_at(
                        event_coords,
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &bounds,
                    ),
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
                        &event_coords,
                    ),
                    _ => (),
                }
            }
            ModifyEventType::Right(relative_pos) => {}
            ModifyEventType::Middle(relative_pos) => {}
            ModifyEventType::Move(relative_pos) => {
                let event_coords = relative_pos.to_bevy_coords();

                match current_target.get() {
                    ModificationTarget::Cutting => cut_sticks(
                        event_coords,
                        &mut stick_query,
                        &params.p0(),
                        &mut commands,
                        &camera,
                    ),
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

fn spawn_point(
    event_coords: Vec2,
    commands: &mut Commands,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
) {
    let parent_point = Point::new(event_coords.extend(0.), event_coords.extend(0.), false);

    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(parent_point.position) //.extend(0.)
            .with_scale(Vec3::splat(0.025)),
        parent_point,
    ));
}

fn lock_affected_points(
    event_coords: Vec2,
    points: &mut Query<(&mut MeshMaterial3d<StandardMaterial>, &mut Point)>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    for (mut material, mut pt) in points {
        if event_coords.extend(0.).distance(pt.position) <= MODIFICATION_RADIUS {
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

fn cut_sticks(
    event_coords: Vec2,
    sticks: &mut Query<(Entity, &mut Stick)>,
    points: &Query<&Point>,
    commands: &mut Commands,
    camera: &Query<&GlobalTransform, With<Camera3d>>,
) {
    for (entity, stick) in sticks {
        if let Ok([p1, p2]) = points.get_many([stick.point1, stick.point2]) {
            // Collect every point along the line to see if the mouse is interacting with it anywhere
            let camera = match camera.get_single() {
                Ok(camera) => camera,
                Err(_) => return,
            };
            let projected_p1 = screen_projection(p1.position, camera);
            let projected_p2 = screen_projection(p2.position, camera);

            let sample_points =
                sample_points_along_line(projected_p1, projected_p2, MODIFICATION_RADIUS);
            for point in sample_points {
                if event_coords.distance(point) <= MODIFICATION_RADIUS {
                    // Remove this line from the simulation
                    commands.entity(entity).despawn();
                    break;
                }
            }
        }
    }
}

/// This function takes in a point in 3D space and projects it into screen-space (Z=0)
fn screen_projection(absolute_point: Vec3, viewing_plane: &GlobalTransform) -> Vec2 {
    // The viewing plane's origin
    let origin_position = viewing_plane.translation();

    // Define the normal vector of the plane
    let origin_norm = viewing_plane.rotation().mul_vec3(Vec3::Z).normalize();

    // Calculate the projection of the absolute point into the relative coordinate frame of the viewing plane
    let projected_point =
        absolute_point - (origin_norm.dot(absolute_point - origin_position)) * origin_norm;

    projected_point.truncate()
}

fn sample_points_along_line(start: Vec2, end: Vec2, spacing: f32) -> Vec<Vec2> {
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

fn connect_points(
    event_coords: Vec2,
    line_query: &mut Query<(Entity, &mut LineConnections)>,
    points: &Query<(Entity, &Point)>,
    commands: &mut Commands,
    stick_mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
) {
    // Ideally there is only one line at any point
    let (_, mut line) = match line_query.get_single_mut() {
        Ok(query_item) => query_item,
        Err(_) => {
            return;
        }
    };

    // Find, optionally, the point that is at the event coordinates
    for (entity, point) in points {
        if event_coords.extend(0.).distance(point.position) <= MODIFICATION_RADIUS {
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

fn spawn_square_at(
    event_coords: Vec2,
    mut commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let point_material = materials.add(StandardMaterial::from(Color::WHITE));
    let stick_material = materials.add(StandardMaterial::from(Color::srgba(1., 1., 1., 0.5)));

    spawn_square(
        &mut commands,
        &mut meshes,
        point_material,
        stick_material,
        event_coords.extend(0.),
    );
}

fn spawn_rope_at(
    event_coords: Vec2,
    mut commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
    bounds: &Res<SimulationBounds>,
) {
    let point_material = materials.add(StandardMaterial::from(Color::WHITE));
    let stick_material = materials.add(StandardMaterial::from(Color::srgba(1., 1., 1., 0.5)));

    spawn_rope(
        &mut commands,
        &mut meshes,
        &mut materials,
        point_material,
        stick_material,
        bounds,
        event_coords.extend(0.),
    );
}
