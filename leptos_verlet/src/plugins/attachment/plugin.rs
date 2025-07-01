use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};

use crate::{
    plugins::{render::plugin::FrameComparison, schedule::plugin::SimulationCycle},
    prelude::Point,
};

#[derive(Component)]
pub struct AttachmentPoint(pub u64);

#[derive(Component, Clone, Copy, Debug)]
pub struct MeshOffset {
    pub translation: Vec3,
    pub rotation: Quat,
    pub anchors: [Vec3; 3],
}

pub struct AttachmentPlugin;
impl Plugin for AttachmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            reposition_models.in_set(SimulationCycle::Preparation3),
        );
    }
}

pub fn reposition_models(
    points: Query<
        (Entity, &AttachmentPoint, &Transform, &Point),
        (With<Point>, Without<SceneRoot>),
    >,
    mut meshes: Query<
        (&AttachmentPoint, &mut Transform, &mut MeshOffset),
        (With<SceneRoot>, Without<Point>),
    >,
    state: Res<FrameComparison>,
    mut initialized: Local<HashSet<u64>>, // track which attachments have their anchors set
) {
    // Build a map of current point positions by attachment ID
    let mut current_map: HashMap<u64, Vec<(Entity, Vec3)>> = HashMap::new();
    for (entity, ap, _tf, pt) in points.iter() {
        current_map
            .entry(ap.0)
            .or_default()
            .push((entity, pt.position));
    }

    // Track which mesh IDs we actually see this frame
    let mut seen_mesh_ids: Vec<u64> = Vec::new();

    for (ap, mut mesh_tf, mut offset) in meshes.iter_mut() {
        let id = ap.0;
        seen_mesh_ids.push(id);

        // ONCE: populate the 3 anchors if exactly three points are present
        if !initialized.contains(&id) {
            if let Some(vec) = current_map.get(&id) {
                if vec.len() == 3 {
                    let mut sorted = vec.clone();
                    sorted.sort_by_key(|(e, _)| *e);
                    offset.anchors = [sorted[0].1, sorted[1].1, sorted[2].1];
                }
                initialized.insert(id);
            }
            // skip any alignment until anchors are assigned
            continue;
        }

        // If nothing moved this frame, skip alignment entirely
        if !state.changed {
            continue;
        }

        // Fetch the current points again (they must still be present)
        if let Some(vec) = current_map.get(&id) {
            match vec.len() {
                3 => {
                    // 3-point alignment
                    let mut sorted = vec.clone();
                    sorted.sort_by_key(|(e, _)| *e);
                    let p0 = sorted[0].1;
                    let p1 = sorted[1].1;
                    let p2 = sorted[2].1;

                    let from_refs = [&offset.anchors[0], &offset.anchors[1], &offset.anchors[2]];
                    let to_refs = [&p0, &p1, &p2];

                    let (delta_rot, delta_trans) = compute_rigid_transform(from_refs, to_refs);

                    // apply above delta to the original offset transform
                    let new_rot = (delta_rot * offset.rotation).normalize();
                    let new_pos = delta_trans + new_rot * offset.translation;

                    mesh_tf.rotation = new_rot;
                    mesh_tf.translation = new_pos;
                }
                1 => {
                    // single-point fallback: place mesh at point + its local offset
                    let p = vec[0].1;
                    mesh_tf.rotation = offset.rotation;
                    mesh_tf.translation = p + offset.rotation * offset.translation;
                }
                _ => {
                    // 0 or 2 points: revert to original mesh transform
                    mesh_tf.rotation = offset.rotation;
                    mesh_tf.translation = offset.translation;
                }
            }
        }
    }

    // Clean up: if any IDs were initialized but no longer have a mesh, remove them
    initialized.retain(|&id| seen_mesh_ids.contains(&id));
}

pub fn compute_rigid_transform(
    source_points: [&Vec3; 3],
    target_points: [&Vec3; 3],
) -> (Quat, Vec3) {
    // compute centroids of source and target triangles
    let source_centroid = (*source_points[0] + *source_points[1] + *source_points[2]) / 3.0;
    let target_centroid = (*target_points[0] + *target_points[1] + *target_points[2]) / 3.0;

    // construct one reference edge and its plane-normal for each triangle
    let from_edge = (*source_points[1] - *source_points[0]).normalize();
    let from_normal = from_edge
        .cross(*source_points[2] - *source_points[0])
        .normalize();
    let to_edge = (*target_points[1] - *target_points[0]).normalize();
    let to_normal = to_edge
        .cross(*target_points[2] - *target_points[0])
        .normalize();

    // rotate source normal to match target normal (align planes)
    let align_normals = Quat::from_rotation_arc(from_normal, to_normal).normalize();

    // rotate the source edge into the plane, then align that edge to target edge (roll)
    let rotated_from_edge = align_normals * from_edge;
    let align_edges = Quat::from_rotation_arc(rotated_from_edge, to_edge).normalize();

    // combine rotations and compute translation to align centroids
    let rotation = align_edges * align_normals;
    let translation = target_centroid - rotation * source_centroid;
    (rotation, translation)
}
