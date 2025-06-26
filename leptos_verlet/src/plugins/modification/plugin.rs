use bevy::prelude::*;

use crate::{
    core::{container_bounds::SimulationBounds, parameters::Stick},
    objects::{cloth::spawn_cloth, cube::spawn_cube, rope::spawn_rope, square::spawn_square},
    plugins::{
        info::plugin::ActiveInfoTarget,
        modification::utils::{
            cut_sticks, lock_affected_points, perge_info_target, point_info, purge_line,
            ray_coords_at, spawn_stick,
        },
        play_state::plugin::SimulationPlayState,
        schedule::plugin::SimulationCycle,
    },
    prelude::{MaterialType, MeshType, Point},
};

pub struct ModificationPlugin;
impl Plugin for ModificationPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ModificationTarget>().add_systems(
            Update,
            (handle_target_change, handle_modification_event)
                .chain()
                .in_set(SimulationCycle::Preparation),
        );
    }
}

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
        Query<(Entity, &Point), With<ActiveInfoTarget>>,
    )>,
    mut next_target: ResMut<NextState<ModificationTarget>>,
    mut line_query: Query<(Entity, &mut LineConnections)>,
    bounds: Res<SimulationBounds>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    let point_material = MaterialType::Color([1., 1., 1., 1.]);
    let stick_material = MaterialType::Color([1., 1., 1., 0.5]);

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
                        point.spawn(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            MeshType::Sphere,
                            MaterialType::Color([1., 1., 1., 1.]),
                        );
                    }
                    ModificationTarget::Line => spawn_stick(
                        ray,
                        &mut line_query,
                        &params.p2(),
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        stick_material.clone(),
                    ),
                    ModificationTarget::Lock => {
                        lock_affected_points(ray, &mut params.p1(), &mut materials);
                    }
                    ModificationTarget::Cut => next_target.set(ModificationTarget::Cutting),
                    ModificationTarget::SpawnSquare => {
                        spawn_square(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            point_material.clone(),
                            stick_material.clone(),
                            view_plane_world_pos,
                        );
                    }
                    ModificationTarget::SpawnRope => {
                        spawn_rope(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            point_material.clone(),
                            stick_material.clone(),
                            &bounds,
                            view_plane_world_pos,
                        );
                    }
                    ModificationTarget::SpawnCloth => spawn_cloth(
                        &mut commands,
                        &mut meshes,
                        point_material.clone(),
                        stick_material.clone(),
                        &mut materials,
                        &bounds,
                    ),
                    ModificationTarget::SpawnCube => spawn_cube(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        point_material.clone(),
                        stick_material.clone(),
                        &view_plane_world_pos,
                    ),
                    ModificationTarget::PointInfo => {
                        // perge existing info targets
                        perge_info_target(&params.p3(), &mut commands);
                        point_info(ray, &params.p2(), &mut commands)
                    }
                    _ => (),
                }
            }
            ModifyEventType::Right(_relative_pos) => {}
            ModifyEventType::Middle(_relative_pos) => {}
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
