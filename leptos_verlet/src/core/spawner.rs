use std::hash::{DefaultHasher, Hash, Hasher};

use bevy::{prelude::*, utils::HashMap};
use web_sys::wasm_bindgen::JsValue;

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

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub enum MeshType {
    Sphere,
    Cuboid,
    Cylinder,
}
impl From<MeshType> for Mesh {
    fn from(descriptor: MeshType) -> Self {
        match descriptor {
            MeshType::Sphere => Sphere::default().into(),
            MeshType::Cuboid => Cuboid::default().into(),
            MeshType::Cylinder => Cylinder::default().into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MaterialType {
    Color([f32; 4]), // RGBA color
}
impl Eq for MaterialType {}
impl Hash for MaterialType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            MaterialType::Color(rgba) => {
                // distinguish this variant in the hash
                0u8.hash(state);
                // hash each float’s bit pattern
                for &c in rgba {
                    c.to_bits().hash(state);
                }
            }
        }
    }
}
impl From<MaterialType> for StandardMaterial {
    fn from(descriptor: MaterialType) -> Self {
        match descriptor {
            MaterialType::Color([r, g, b, a]) => StandardMaterial {
                base_color: Color::srgba(r, g, b, a),
                alpha_mode: AlphaMode::Blend,
                ..Default::default()
            },
        }
    }
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
    /// The model_name for any imported model to be attached to this point.
    pub attachment: Option<String>,
    /// How to scale the generated point visually
    pub point_scale: Vec3,
    /// How to scale the generated sticks visually
    pub connection_scale: Option<Vec<Vec3>>,
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
            point_scale: Vec3::ONE,
            connection_scale: None,
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
    // Cache all MeshType -> Handle<Mesh> and MaterialType -> Handle<StandardMaterial>
    let mut mesh_handles: HashMap<MeshType, Handle<Mesh>> = HashMap::new();
    let mut material_handles: HashMap<MaterialType, Handle<StandardMaterial>> = HashMap::new();

    for spawn_node in &mesh_network {
        // Cache the mesh handle for the point
        mesh_handles
            .entry(spawn_node.point_mesh.clone())
            .or_insert_with(|| meshes.add(Mesh::from(spawn_node.point_mesh.clone())));

        // Cache mesh handles for each connection mesh
        if let Some(connection_mesh_types) = &spawn_node.connection_mesh {
            for mesh_type in connection_mesh_types {
                mesh_handles
                    .entry(mesh_type.clone())
                    .or_insert_with(|| meshes.add(Mesh::from(mesh_type.clone())));
            }
        }

        // Cache the material handle for the point
        material_handles
            .entry(spawn_node.point_material.clone())
            .or_insert_with(|| {
                materials.add(StandardMaterial::from(spawn_node.point_material.clone()))
            });

        // Cache material handles for each connection material
        if let Some(connection_material_types) = &spawn_node.connection_material {
            for material_type in connection_material_types {
                material_handles
                    .entry(material_type.clone())
                    .or_insert_with(|| {
                        materials.add(StandardMaterial::from(material_type.clone()))
                    });
            }
        }
    }

    // Spawn points and record their Entity IDs
    let mut spawned_entities: Vec<SpawnedEntity> = Vec::new();
    for spawn_node in &mesh_network {
        let point_mesh_handle = mesh_handles[&spawn_node.point_mesh].clone();
        let point_material_handle = material_handles[&spawn_node.point_material].clone();

        let mut spawn_command = commands.spawn((
            Mesh3d(point_mesh_handle),
            MeshMaterial3d(point_material_handle),
            Transform::from_translation(spawn_node.point.position)
                .with_scale(spawn_node.point_scale * spawn_node.point_size),
            spawn_node.point.clone(),
        ));

        // If this point is marked as an attachment point, add that component
        if let Some(attachment_data) = &spawn_node.attachment {
            let mut hasher = DefaultHasher::new();
            attachment_data.hash(&mut hasher);
            spawn_command.insert(AttachmentPoint(hasher.finish()));
        }

        spawned_entities.push(SpawnedEntity::new(
            spawn_node.point.position,
            spawn_command.id(),
        ));
    }

    // Spawn sticks (connections)
    for (parent_index, spawn_node) in mesh_network.iter().enumerate() {
        // Unwrap optional connection data or skip if none
        let connection_positions = if let Some(positions) = &spawn_node.connection {
            positions
        } else {
            continue;
        };
        let connection_material_types = if let Some(materials_vec) = &spawn_node.connection_material
        {
            materials_vec
        } else {
            continue;
        };
        let connection_mesh_types = if let Some(mesh_types) = &spawn_node.connection_mesh {
            mesh_types
        } else {
            continue;
        };
        let connection_size_values = if let Some(size_values) = &spawn_node.connection_size {
            size_values
        } else {
            continue;
        };
        let connection_scale_values = if let Some(scale_values) = &spawn_node.connection_scale {
            scale_values
        } else {
            // Check if connections exist and just default to all ONES
            let mut scale_values = Vec::new();
            for _ in connection_positions {
                scale_values.push(Vec3::ONE)
            }
            &scale_values.clone()
        };

        // Sanity checks on lengths
        assert!(
            connection_positions.len() == connection_material_types.len(),
            "Material count must match connections"
        );
        assert!(
            connection_positions.len() == connection_mesh_types.len(),
            "Mesh count must match connections"
        );
        assert!(
            connection_positions.len() == connection_size_values.len(),
            "Size count must match connections"
        );
        assert!(
            connection_positions.len() == connection_scale_values.len(),
            "Scale count must match connections"
        );

        for (connection_index, &connection_position) in connection_positions.iter().enumerate() {
            // Find the entity for this connection position
            let connected_entity_option = spawned_entities
                .iter()
                .find(|entity_info| entity_info.position == connection_position);

            if let Some(connected_entity) = connected_entity_option {
                // Ensure each stick is only spawned once (parent_index < child_index)
                let child_index = spawned_entities
                    .iter()
                    .position(|entity_info| entity_info == connected_entity)
                    .unwrap();
                if parent_index > child_index {
                    continue;
                }

                let start_position = spawn_node.point.position;
                let end_position = connected_entity.position;
                let direction_vector = end_position - start_position;
                let rotation_quat = Quat::from_rotation_arc(Vec3::X, direction_vector.normalize());

                // Lookup cached handles
                let stick_mesh_handle =
                    mesh_handles[&connection_mesh_types[connection_index]].clone();
                let stick_material_handle =
                    material_handles[&connection_material_types[connection_index]].clone();

                commands.spawn((
                    Mesh3d(stick_mesh_handle),
                    MeshMaterial3d(stick_material_handle),
                    Transform {
                        translation: (start_position + end_position) * 0.5,
                        rotation: rotation_quat,
                        scale: Vec3::new(
                            direction_vector.length(),
                            connection_size_values[connection_index],
                            connection_size_values[connection_index],
                        ) * connection_scale_values[connection_index],
                    },
                    Stick::new(
                        spawned_entities[parent_index].entity,
                        connected_entity.entity,
                        direction_vector.length(),
                    ),
                ));
            } else {
                web_sys::console::log_1(&JsValue::from_str(&format!(
                    "Requested node position doesn't exist: {:?}",
                    connection_position
                )));
            }
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
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
    }
}

fn _mesh_from_descriptor(descriptor: &MeshType, meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
    match descriptor {
        MeshType::Sphere => meshes.add(Sphere::default()),
        MeshType::Cuboid => meshes.add(Cuboid::default()),
        MeshType::Cylinder => meshes.add(Cylinder::default()),
    }
}
