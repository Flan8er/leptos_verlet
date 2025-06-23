use bevy::prelude::*;

use crate::core::parameters::{Point, Stick};

#[derive(Clone, Debug)]
pub struct SpawnNode {
    /// The point to spawn.
    pub point: Point,
    /// A list of connections this point should share with other points.
    pub connection: Option<Vec<Vec3>>,
    /// The material of the point. Note, any 'locked' point will be displayed as red.
    pub point_material: Handle<StandardMaterial>,
    /// A specified material for each connection.
    pub connection_material: Option<Vec<Handle<StandardMaterial>>>,
    /// The mesh of the point.
    pub point_mesh: Handle<Mesh>,
    /// A specified mesh for each connection.
    pub connection_mesh: Option<Vec<Handle<Mesh>>>,
    /// The diameter of the point.
    pub point_size: f32,
    /// The thickness of the connection.
    pub connection_size: Option<Vec<f32>>,
}

#[derive(Clone, Debug, PartialEq)]
struct SpawnedEntity {
    position: Vec3,
    entity: Entity,
}
impl SpawnedEntity {
    fn new(position: Vec3, entity: Entity) -> Self {
        Self { position, entity }
    }
}

pub fn spawner(mesh_network: Vec<SpawnNode>, commands: &mut Commands) {
    // An square can be defined as:
    // [
    // {
    //  (0, 0, 0), [(0, 1, 0)]
    // },
    // {
    //  (0, 1, 0), [(1, 1, 0), (1, 0, 0)]
    // },
    // {
    //  (1, 1, 0), [(1, 0, 0)]
    // },
    // {
    //  (1, 0, 0), [(0, 0, 0), (0, 1, 0)]
    // }
    // ]

    // Create a vector to keep track of the spawned points and their locations
    let mut spawned_entities: Vec<SpawnedEntity> = Vec::new();

    // Spawn all the nodes in the mesh network
    for spawn_node in mesh_network.iter() {
        // Spawn the point at the requested location
        let spawned_point = commands
            .spawn((
                Mesh3d(spawn_node.point_mesh.clone()),
                MeshMaterial3d(spawn_node.point_material.clone()),
                Transform::from_translation(spawn_node.point.position)
                    .with_scale(Vec3::splat(spawn_node.point_size)),
                spawn_node.point.clone(),
            ))
            .id();

        // Keep track of the entity and position for later possible insertion into a Stick
        spawned_entities.push(SpawnedEntity::new(spawn_node.point.position, spawned_point));
    }

    // Go through and connect all the nodes that requested connection
    for (index, spawn_node) in mesh_network.iter().enumerate() {
        // Ensure all necessary data for a connection exists
        let connections = match &spawn_node.connection {
            Some(connection_vec) => connection_vec,
            None => continue,
        };
        let connection_material = match &spawn_node.connection_material {
            Some(material_vec) => material_vec,
            None => continue,
        };
        let connection_mesh = match &spawn_node.connection_mesh {
            Some(mesh_vec) => mesh_vec,
            None => continue,
        };
        let connection_size = match &spawn_node.connection_size {
            Some(size_vec) => size_vec,
            None => continue,
        };
        assert!(
            connections.len() == connection_material.len(),
            "Error: The number of materials must match the number of requested connections. Each connection must provide it's own material."
        );
        assert!(
            connections.len() == connection_mesh.len(),
            "Error: The number of meshes must match the number of requested connections. Each connection must provide it's own mesh."
        );
        assert!(
            connections.len() == connection_size.len(),
            "Error: The number of size constraints must match the number of requested connections. Each connection must provide it's own size."
        );

        // For each of the requested connections find the point at the requested position
        for (j, connection_request) in connections.iter().enumerate() {
            if let Some(connecting_entity) = spawned_entities
                .iter()
                .find(|node| &node.position == connection_request)
            {
                // Check if this connection has already been made
                if let Some((idx, _)) = spawned_entities
                    .iter()
                    .enumerate()
                    .find(|(_idx, entity)| entity == &connecting_entity)
                {
                    if index > idx {
                        // The stick has already been spawned by a previous iteration
                        continue;
                    }
                }

                // Join the spawn node with the connecting entity
                // Create a stick joining the parent to the child
                let spacial_point_1 = spawn_node.point.position;
                let spacial_point_2 = connecting_entity.position;

                // Determine the objects rotation quaternion
                let diff = spacial_point_2 - spacial_point_1;
                let rot = Quat::from_rotation_arc(Vec3::X, diff.normalize());

                // Spawn the stick linking the two points
                commands.spawn((
                    Mesh3d(connection_mesh[j].clone()),
                    MeshMaterial3d(connection_material[j].clone()),
                    Transform {
                        // Translate based off the midpoint of the line
                        translation: (spacial_point_1 + spacial_point_2) * 0.5,
                        rotation: rot,
                        scale: Vec3::new(
                            diff.length(),
                            connection_size[j].clone(),
                            connection_size[j].clone(),
                        ),
                    },
                    Stick::new(
                        spawned_entities[index].entity,
                        connecting_entity.entity,
                        diff.length(),
                    ),
                ));
            } else {
                panic!(
                    "The requested node position doesn't exist: {:#?}",
                    connection_request
                );
            };
        }
    }
}
