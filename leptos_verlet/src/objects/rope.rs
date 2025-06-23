use bevy::prelude::*;

use crate::{
    core::parameters::Point,
    interaction::window_bounds::SimulationBounds,
    objects::spawner::{SpawnNode, spawner},
};

const POINT_SIZE: f32 = 0.025; // m (0.025m == 25mm)
const STICK_SIZE: f32 = 0.01; // m (0.025m == 25mm)
/// Smaller stick length will result in a denser rope
const STICK_LENGTH: f32 = 0.025; // m
const DROP_ANGLE: f32 = 35.; // deg
const ROPE_LENGTH: f32 = 1.5; // m

pub fn spawn_rope(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    point_material: Handle<StandardMaterial>,
    stick_material: Handle<StandardMaterial>,
    bounds: &Res<SimulationBounds>,
    position: Vec3,
) {
    let point_mesh = meshes.add(Sphere::default());
    let stick_mesh = meshes.add(Cuboid::default());

    // red “locked” material for the root node
    let locked_material = materials.add(StandardMaterial::from(Color::srgb(1.0, 0.0, 0.0)));

    // how many sticks (and thus how many extra points)
    let sticks_tot = (ROPE_LENGTH / STICK_LENGTH).floor() as usize;

    // build the chain of point‐positions, respecting bounds of simulation
    let theta = DROP_ANGLE.to_radians();
    let mut positions = Vec::with_capacity(sticks_tot + 1);
    positions.push(position);
    for _ in 0..sticks_tot {
        let prev = *positions.last().unwrap();
        let mut next =
            prev + Vec3::new(-STICK_LENGTH * theta.sin(), STICK_LENGTH * theta.cos(), 0.0);

        // clamp X into [–width/2, +width/2]
        let half_w = bounds.width / 2.0;
        if next.x <= -half_w {
            let cx = -half_w + 0.001;
            let dy = cx - next.x;
            next = Vec3::new(cx, next.y + dy, next.z);
        } else if next.x >= half_w {
            let cx = half_w - 0.001;
            let dy = next.x - cx;
            next = Vec3::new(cx, next.y + dy, next.z);
        }

        positions.push(next);
    }

    // build adjacency for a linear chain (bidirectional)
    let mut adj = vec![Vec::new(); positions.len()];
    for i in 0..positions.len() - 1 {
        adj[i].push(i + 1);
        adj[i + 1].push(i);
    }

    // turn positions + adj into Vec<SpawnNode>
    let mut mesh_network = Vec::with_capacity(positions.len());
    for i in 0..positions.len() {
        let pos = positions[i];
        let locked = i == 0;
        let neighbors = &adj[i];

        mesh_network.push(SpawnNode {
            point: Point::new(pos, pos, locked),
            connection: Some(neighbors.iter().map(|&j| positions[j]).collect()),
            point_material: if locked {
                locked_material.clone()
            } else {
                point_material.clone()
            },
            point_mesh: point_mesh.clone(),
            point_size: POINT_SIZE,
            connection_mesh: Some(vec![stick_mesh.clone(); neighbors.len()]),
            connection_material: Some(vec![stick_material.clone(); neighbors.len()]),
            connection_size: Some(vec![STICK_SIZE; neighbors.len()]),
        });
    }

    spawner(mesh_network, commands);
}
