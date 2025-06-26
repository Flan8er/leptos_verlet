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
    /// The value indicating the maximum number of unchanged frames that will trigger a rerender.
    /// The data should still render, but not unnecessarily.
    pub max_unchanged: u32,
}
impl Default for FrameComparison {
    fn default() -> Self {
        Self {
            frames_since: 0,
            changed: true,
            max_unchanged: 120,
        }
    }
}

fn render_points(mut query: Query<(&Point, &mut Transform)>, state: Res<FrameComparison>) {
    // Dont rerender if the state hasnt changed
    if !state.changed {
        return;
    }

    for (point, mut transform) in &mut query {
        transform.translation = point.position;
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

    // 1) Update/rotate each stick…
    for (stick, mut stick_tf) in stick_tf_q.iter_mut() {
        let p1 = point_pos_q.get(stick.point1).unwrap();
        let p2 = point_pos_q.get(stick.point2).unwrap();

        let mid = (p1.position + p2.position) * 0.5;
        let rot = Quat::from_rotation_arc(Vec3::X, (p2.position - p1.position).normalize());

        stick_tf.translation = mid;
        stick_tf.rotation = rot;

        // 2) …and immediately drive its endpoints’ rotations:
        if let Ok(mut pt1_tf) = point_tf_q.get_mut(stick.point1) {
            pt1_tf.rotation = rot;
        }
        if let Ok(mut pt2_tf) = point_tf_q.get_mut(stick.point2) {
            pt2_tf.rotation = rot;
        }
    }

    // (Optionally, if you also want to propagate position changes here instead
    // of in a separate system, you could loop point_tf_q and set translations.)
}
