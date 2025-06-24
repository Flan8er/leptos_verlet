use bevy::prelude::*;

use crate::{
    core::parameters::Point,
    objects::spawner::{SpawnNode, spawner},
};

const HALF_SIZE: f32 = 0.225;
const POINT_SIZE: f32 = 0.025; // m (0.025m == 25mm)
const STICK_SIZE: f32 = 0.01; // m (0.025m == 25mm)

pub fn spawn_square(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    point_material: Handle<StandardMaterial>,
    stick_material: Handle<StandardMaterial>,
    position: Vec3,
) {
    let stick_mesh = meshes.add(Cuboid::default());
    let point_mesh = meshes.add(Sphere::default());

    // corner offsets for a unit-square
    let offsets = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)];

    // compute actual corner positions
    let corners: Vec<Vec3> = offsets
        .iter()
        .map(|&(dx, dy)| position + Vec3::new(dx * HALF_SIZE, dy * HALF_SIZE, 0.0))
        .collect();

    // define connectivity as index-pairs
    //    perimeter + one diagonal
    let edges = [(0, 1), (1, 2), (2, 3), (3, 0), (1, 3)];

    // build adjacency list: for each corner `i` collect all `j` it connects to
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); corners.len()];
    for &(i, j) in &edges {
        adj[i].push(j);
        adj[j].push(i);
    }

    // build the SpawnNode list
    let mesh_network: Vec<SpawnNode> = corners
        .clone()
        .into_iter()
        .enumerate()
        .map(|(i, corner_pos)| {
            let neighbors = &adj[i];
            // for each neighbor, we'll supply its absolute position ...
            let connection = Some(neighbors.iter().map(|&j| corners[j]).collect());
            // ... and exactly one mesh / material / size per connection
            let connection_mesh = Some(vec![stick_mesh.clone(); neighbors.len()]);
            let connection_material = Some(vec![stick_material.clone(); neighbors.len()]);
            let connection_size = Some(vec![STICK_SIZE; neighbors.len()]);

            SpawnNode {
                point: Point::new(corner_pos, corner_pos, false),
                connection,
                point_material: point_material.clone(),
                connection_material,
                point_mesh: point_mesh.clone(),
                connection_mesh,
                point_size: POINT_SIZE,
                connection_size,
            }
        })
        .collect();

    spawner(mesh_network, commands);
}
