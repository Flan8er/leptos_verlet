use bevy::prelude::*;

use crate::{
    core::parameters::{Point, Stick},
    plugins::schedule::plugin::SimulationCycle,
};

pub struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FrameComparison::default()).add_systems(
            Update,
            (render_points, render_points_and_sticks)
                .chain()
                .in_set(SimulationCycle::Render),
        );
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Resource)]
pub struct FrameComparison {
    /// Represents the number of frames that have ellapsed since the last render.
    pub frames_since: u32,
    /// Has this frame changed and needs to be rendered?
    pub changed: bool,
}
impl Default for FrameComparison {
    fn default() -> Self {
        Self {
            frames_since: 0,
            changed: true,
        }
    }
}

fn render_points(mut query: Query<(&mut Point, &mut Transform)>, state: Res<FrameComparison>) {
    // Dont rerender if the state hasnt changed
    if !state.changed {
        return;
    }

    for (mut point, mut transform) in &mut query {
        transform.translation = point.position;
        point.rendered_position = point.position;
        point.previously_rendered_position = point.prev_position;
    }
}

/// Render's sticks based off the new point translations but also applies rotation to the points based off it's sticks rotation.
fn render_points_and_sticks(
    point_pos_q: Query<&Point>,
    mut point_tf_q: Query<&mut Transform, (With<Point>, Without<Stick>)>,
    mut stick_tf_q: Query<(&Stick, &mut Transform), (With<Stick>, Without<Point>)>,
    state: Res<FrameComparison>,
) {
    if !state.changed {
        return;
    }

    // Update/rotate each stick…
    for (stick, mut stick_tf) in stick_tf_q.iter_mut() {
        let p1 = match point_pos_q.get(stick.point1) {
            Ok(point) => point,
            Err(_) => continue,
        };
        let p2 = match point_pos_q.get(stick.point2) {
            Ok(point) => point,
            Err(_) => continue,
        };

        let mid = (p1.position + p2.position) * 0.5;
        let rot = Quat::from_rotation_arc(Vec3::X, (p2.position - p1.position).normalize());

        stick_tf.translation = mid;
        stick_tf.rotation = rot;

        // …and immediately drive its endpoints’ rotations:
        if let Ok(mut pt1_tf) = point_tf_q.get_mut(stick.point1) {
            pt1_tf.rotation = rot;
        }
        if let Ok(mut pt2_tf) = point_tf_q.get_mut(stick.point2) {
            pt2_tf.rotation = rot;
        }
    }
}
