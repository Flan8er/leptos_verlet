use bevy::prelude::*;

use crate::core::{container_bounds::window_listener, parameters::SimulationSettings};

pub struct StartupPlugin;
impl Plugin for StartupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_ui).chain())
            .add_systems(Update, (window_listener,));
    }
}

fn setup_ui(mut commands: Commands, sim_settings: Res<SimulationSettings>) {
    let camera_position = sim_settings.camera_position;
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: sim_settings.camera_fov,
            ..default()
        }),
        Transform::from_xyz(camera_position.x, camera_position.y, camera_position.z),
    ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 10_000.0, // sunlight intensity in lux
            color: Color::WHITE,
            ..default()
        },
        Transform {
            translation: Vec3::new(7.5, 7., 5.), // position is arbitrary for directional light
            rotation: Quat::from_rotation_arc(
                Vec3::NEG_Z, // default direction it points toward
                // Vec3::new(-1.0, -1.0, -1.0).normalize(), // sun direction
                Vec3::new(-7.5, -7., -5.).normalize(),
            ),
            ..default()
        },
    ));

    // commands.insert_resource(AmbientLight {
    //     color: Color::WHITE,
    //     brightness: 500.,
    // });
}
