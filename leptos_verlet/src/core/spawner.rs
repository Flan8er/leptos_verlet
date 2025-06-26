use std::hash::{DefaultHasher, Hash, Hasher};

use bevy::prelude::*;

use crate::{
    core::parameters::{Point, Stick},
    plugins::attachment::plugin::AttachmentPoint,
};

#[derive(Debug, Clone, Resource)]
/// The buffer used to collect any mesh network reqested to be spawned.
pub struct SpawnBuffer {
    pub buffer: Vec<SpawnRequest>,
}
impl Default for SpawnBuffer {
    fn default() -> Self {
        Self { buffer: Vec::new() }
    }
}

#[derive(Event, Clone, Debug, PartialEq)]
pub struct SpawnRequest {
    pub mesh_network: Vec<SpawnNode>,
}
impl SpawnRequest {
    pub fn new(mesh_network: Vec<SpawnNode>) -> Self {
        Self { mesh_network }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MeshType {
    Sphere,
    Cuboid,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MaterialType {
    Color([f32; 4]), // RGBA color
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpawnNode {
    /// The point to spawn.
    pub point: Point,
    /// A list of connections this point should share with other points.
    pub connection: Option<Vec<Vec3>>,
    /// The material of the point. Note, any 'locked' point will be displayed as red.
    pub point_material: MaterialType,
    /// A specified material for each connection.
    pub connection_material: Option<Vec<MaterialType>>,
    /// The mesh of the point.
    pub point_mesh: MeshType,
    /// A specified mesh for each connection.
    pub connection_mesh: Option<Vec<MeshType>>,
    /// The diameter of the point.
    pub point_size: f32,
    /// The thickness of the connection.
    pub connection_size: Option<Vec<f32>>,
    pub attachment: Option<String>,
}
impl Default for SpawnNode {
    fn default() -> Self {
        Self {
            point: Point::new(Vec3::ZERO, Vec3::ZERO, false),
            connection: None,
            point_material: MaterialType::Color([1., 1., 1., 1.]),
            connection_material: None,
            point_mesh: MeshType::Sphere,
            connection_mesh: None,
            point_size: 0.025,
            connection_size: None,
            attachment: None,
        }
    }
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

pub fn spawner(
    mesh_network: Vec<SpawnNode>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Create a vector to keep track of the spawned points and their locations
    let mut spawned_entities: Vec<SpawnedEntity> = Vec::new();

    // Spawn all the nodes in the mesh network
    for spawn_node in mesh_network.iter() {
        // Spawn the point at the requested location
        let mut cmd = commands.spawn((
            Mesh3d(mesh_from_descriptor(&spawn_node.point_mesh, meshes)),
            MeshMaterial3d(material_from_descriptor(
                &spawn_node.point_material,
                materials,
            )),
            Transform::from_translation(spawn_node.point.position)
                .with_scale(Vec3::splat(spawn_node.point_size)),
            spawn_node.point.clone(),
        ));

        // If the point is marked as an attachment point, spawn that in as well
        if let Some(attachment) = spawn_node.attachment.clone() {
            let mut hasher = DefaultHasher::new();
            attachment.hash(&mut hasher);

            cmd.insert(AttachmentPoint(hasher.finish()));
        }

        let spawned_point = cmd.id();

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
                    Mesh3d(mesh_from_descriptor(&connection_mesh[j], meshes)),
                    MeshMaterial3d(material_from_descriptor(&connection_material[j], materials)),
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

pub fn material_from_descriptor(
    descriptor: &MaterialType,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Handle<StandardMaterial> {
    match descriptor {
        MaterialType::Color([r, g, b, a]) => materials.add(StandardMaterial {
            base_color: Color::srgba(*r, *g, *b, *a),
            ..default()
        }),
    }
}

fn mesh_from_descriptor(descriptor: &MeshType, meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
    match descriptor {
        MeshType::Sphere => meshes.add(Sphere::default()),
        MeshType::Cuboid => meshes.add(Cuboid::default()),
    }
}
