use bevy::prelude::*;

use crate::{
    core::{
        parameters::{Point, SimulationSettings},
        spawner::{SpawnNode, spawner},
    },
    prelude::{MaterialType, MeshType},
};

const HALF_SIZE: f32 = 0.225;

pub fn spawn_cube(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    point_material: MaterialType,
    stick_material: MaterialType,
    center: &Vec3,
    sim_settings: &Res<SimulationSettings>,
) {
    // pick meshes
    let point_mesh = MeshType::Sphere;
    let stick_mesh = MeshType::Cuboid;

    // the eight corners of a unit-cube
    let offsets = [
        (-1., -1., -1.), // 0
        (1., -1., -1.),  // 1
        (1., 1., -1.),   // 2
        (-1., 1., -1.),  // 3
        (-1., -1., 1.),  // 4
        (1., -1., 1.),   // 5
        (1., 1., 1.),    // 6
        (-1., 1., 1.),   // 7
    ];

    let corners: Vec<Vec3> = offsets
        .iter()
        .map(|&(dx, dy, dz)| {
            center + Vec3::new(dx * HALF_SIZE, dy * HALF_SIZE, dz * HALF_SIZE - HALF_SIZE)
        })
        .collect();

    // the 12 cube-edges (undirected), plus one diagonal per face:
    let mut edges = vec![
        // bottom face (dz = –1)
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        // top face (dz = +1)
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        // vertical pillars
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];

    // face diagonals - one per face:
    let face_diagonals = [
        (1, 3), // bottom face
        (5, 7), // top face
        (3, 4), // left  face (x = –1)
        (2, 5), // right face (x = +1)
        (1, 4), // front face (y = –1)
        (2, 7), // back  face (y = +1)
    ];
    edges.extend_from_slice(&face_diagonals);

    // build bidirectional adjacency
    let mut adj = vec![Vec::new(); corners.len()];
    for &(i, j) in &edges {
        adj[i].push(j);
        adj[j].push(i);
    }

    // assemble mesh_network
    let mesh_network: Vec<SpawnNode> = corners
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, pos)| {
            let neigh = &adj[i];
            SpawnNode {
                point: Point::new(pos, pos, false),
                connection: Some(neigh.iter().map(|&j| corners[j]).collect()),
                point_material: point_material.clone(),
                point_mesh: point_mesh.clone(),
                point_size: sim_settings.default_geometry_point_size,
                connection_mesh: Some(vec![stick_mesh.clone(); neigh.len()]),
                connection_material: Some(vec![stick_material.clone(); neigh.len()]),
                connection_size: Some(vec![sim_settings.default_geometry_stick_size; neigh.len()]),
                connection_scale: Some(vec![Vec3::ONE; neigh.len()]),
                ..default()
            }
        })
        .collect();

    spawner(mesh_network, commands, meshes, materials);
}
