use bevy::prelude::*;

use crate::core::{
    parameters::{Point, Stick},
    schedule::SimulationCycle,
};

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

pub struct RenderView;
impl Plugin for RenderView {
    fn build(&self, app: &mut App) {
        app.insert_resource(FrameComparison::default()).add_systems(
            Update,
            (render_points, render_sticks)
                .chain()
                .in_set(SimulationCycle::Render),
        );
    }
}

pub fn render_points(mut query: Query<(&Point, &mut Transform)>, state: Res<FrameComparison>) {
    // Dont rerender if the state hasnt changed
    if !state.changed {
        return;
    }

    for (point, mut transform) in &mut query {
        transform.translation = point.position;
    }
}

pub fn render_sticks(
    point_query: Query<&Point>,
    mut stick_query: Query<(&Stick, &mut Transform)>,
    state: Res<FrameComparison>,
) {
    // Dont rerender if the state hasnt changed
    if !state.changed {
        return;
    }

    for (stick, mut t) in stick_query.iter_mut() {
        // get the two Point components
        let p1 = match point_query.get(stick.point1) {
            Ok(point) => point,
            Err(_) => {
                continue;
            }
        };
        let p2 = match point_query.get(stick.point2) {
            Ok(point) => point,
            Err(_) => continue,
        };

        let a = p1.position;
        let b = p2.position;

        // midpoint
        let mid = (a + b) * 0.5;
        // direction and length
        let diff = b - a;
        // let len = diff.length();

        // write into the Transform
        t.translation = mid;
        // assume your Cuboid is length-1 along +X; rotate X→diff
        t.rotation = Quat::from_rotation_arc(Vec3::X, diff.normalize());
        // stretch X to match the distance; keep Y/Z at 1 (or whatever thickness you like)
        // t.scale = Vec3::new(len, 0.025, 0.025);
    }
}
