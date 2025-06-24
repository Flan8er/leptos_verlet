use bevy::{prelude::*, render::camera::ScalingMode};

use crate::{
    core::parameters::{CAMERA_DISTANCE, CAMERA_FOV, HALF_CAMERA_HEIGHT},
    interaction::window_bounds::SimulationBounds,
    objects::{rope::spawn_rope, square::spawn_square},
};

pub struct StartSimulation;
impl Plugin for StartSimulation {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_ui, load_initial_particles).chain());
    }
}

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: CAMERA_FOV,
            ..default()
        }),
        // Projection::Orthographic(OrthographicProjection {
        //     scaling_mode: ScalingMode::FixedVertical {
        //         viewport_height: *HALF_CAMERA_HEIGHT * 2.,
        //     },
        //     scale: 1.,
        //     ..OrthographicProjection::default_2d()
        // }),
        Transform::from_xyz(0.0, *HALF_CAMERA_HEIGHT, CAMERA_DISTANCE),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 2000.,
    });
}

fn load_initial_particles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    bounds: Res<SimulationBounds>,
) {
    spawn_particles(&mut commands, &mut meshes, &mut materials, &bounds);
}

pub fn spawn_particles(
    mut commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
    bounds: &Res<SimulationBounds>,
) {
    let point_material = materials.add(StandardMaterial::from(Color::WHITE));
    let stick_material = materials.add(StandardMaterial::from(Color::srgba(1., 1., 1., 0.5)));

    spawn_square(
        &mut commands,
        &mut meshes,
        point_material.clone(),
        stick_material.clone(),
        Vec3::new(0., *HALF_CAMERA_HEIGHT, 0.),
    );
    spawn_rope(
        &mut commands,
        &mut meshes,
        &mut materials,
        point_material,
        stick_material,
        bounds,
        Vec3::new(0., *HALF_CAMERA_HEIGHT * 1.75, 0.),
    );
}
