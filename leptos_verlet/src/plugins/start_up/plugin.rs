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
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: sim_settings.camera_fov,
            ..default()
        }),
        Transform {
            translation: sim_settings.camera_position,
            rotation: sim_settings.camera_orientation,
            ..default()
        },
    ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: sim_settings.light_luminosity,
            color: Color::WHITE,
            ..default()
        },
        Transform {
            translation: sim_settings.light_position,
            rotation: sim_settings.light_orientation,
            ..default()
        },
    ));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: sim_settings.ambient_light,
        ..default()
    });
}
