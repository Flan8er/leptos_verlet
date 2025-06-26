use bevy::prelude::*;
use std::collections::HashMap;

use crate::{plugins::schedule::plugin::SimulationCycle, prelude::Point};

#[derive(Component)]
pub struct AttachmentPoint(pub u64);

pub struct AttachmentPlugin;
impl Plugin for AttachmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, reposition_models.in_set(SimulationCycle::Render));
    }
}

fn reposition_models(
    points: Query<(&AttachmentPoint, &Transform), (With<Point>, Without<SceneRoot>)>,
    mut models: Query<(&AttachmentPoint, &mut Transform), (With<SceneRoot>, Without<Point>)>,
) {
    // Collect all point-transforms into a HashMap
    let mut positions = HashMap::new();
    for (ap, tf) in points.iter() {
        // copy only the parts we need
        positions.insert(ap.0, (tf.translation, tf.rotation));
    }

    // For each model, look up its matching point and apply the necessary translations and rotations
    // to get it to realign with its designated attachment point
    for (ap, mut tf) in models.iter_mut() {
        if let Some((pos, rot)) = positions.get(&ap.0) {
            tf.translation = *pos;
            tf.rotation = *rot;
        }
    }
}
