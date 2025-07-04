[![crates.io](https://img.shields.io/crates/v/leptos_verlet.svg)](https://crates.io/crates/leptos_verlet)
[![docs.rs](https://docs.rs/leptos_verlet/badge.svg)](https://docs.rs/leptos_verlet)

# Leptos Verlet

An engine that allows the addition of interactive verlet simulations into any leptos app.

- Spawned objects are interactive through container bounds allowing for a uniquely interactive component.
- A host of prebuilt objects using an agnostic "builder" layer that allows the developer to define and spawn custom objects into the simulation.

![Cloth](https://github.com/user-attachments/assets/f89de049-1c1f-402b-84d5-e8dea601a3db)
![Pendulum](https://github.com/user-attachments/assets/ed8f5c25-766e-41b0-a849-c2462c62f0ad)
![Rope](https://github.com/user-attachments/assets/8b6b1229-d510-4c40-89f6-fcb07aa85ea5)

# Implementation

## Using

```rust
use leptos_verlet::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    let simulation_container = NodeRef::<Div>::new();

    let active_modifier: RwSignal<ModificationTarget> = RwSignal::new(ModificationTarget::None);

    view! {
        <VerletConfigProvider/>

        <main class="w-screen h-screen flex items-center justify-center overflow-hidden relative">
            <ElementPane active_modifier/>
            <InfoModal active_modifier/>

            <div
                node_ref=simulation_container
                class="w-full h-full relative"
            >
                <VerletCanvas parent_element=simulation_container/>

                <MouseMonitor active_modifier/>
            </div>

            <ControlPane active_modifier/>
        </main>
    }
}
```

## Custom Meshes

In the latest ^1.2 update, a new function is exposed to allow custom meshes to be imported and attached to a simulation Point. The mesh will track, follow, and reorient relative to whatever Point it's attached to.

```rust
// Imports and spawns the mesh into the simulation
// Must be ran inside a reactive context
model_loader("/static/monkey.glb", "monkey.glb", 0);

// To attach the mesh to a SpawnNode:
SpawnNode {
    attachment: Some(String::from("monkey.glb")),
    ..default()
}
```

The SpawnNode type also takes in an extra argument: "attachment". This is an optional String where the value is the same model_name used in the model_loader function ("monkey.glb" in the above code). This essentially tells the mesh to follow whatever point it is attached to.

<img width="1765" alt="Screenshot 2025-07-01 at 8 50 10 AM" src="https://github.com/user-attachments/assets/fd9a9d9a-b996-4fa9-8160-b718941d796d" />

## Custom Shapes

Any shape can be created, simulated, and styled using the built in spawner that reads from a Vec of SpawnNode.

```rust
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
}

pub struct SpawnRequest {
    pub mesh_network: Vec<SpawnNode>,
}
```

Shown below is a verbose use case for spawning a square to be used to visualize the mesh network system. As much as feasible, a system should be created for programmatically generating these structures.

The desired vertices of the shape are constructed (below the initial velocity is set to zero by giving the point the same "current position" as "previous position"), and then added to a SpawnNode with the desired connection vertices and mesh/material styling. A mesh_network is then constructed and sent as a spawn request.

```rust
use leptos_verlet::prelude::*;

let spawn_request = expect_context::<SpawnSender>();

let square_size = 0.45;
let point_size = 0.025;
let stick_size = 0.01;
let point_mesh = MeshType::Sphere;
let stick_mesh = MeshType::Cuboid;
let point_material = MaterialType::Color([1., 1., 1., 1.]); // The spawned points will be pure white
let stick_material = MaterialType::Color([1., 1., 1., 0.75]); // The spawned connections will be opaque white

let bottom_left = Vec3::new(-square_size / 2., 0., 0.);
let bottom_right = Vec3::new(square_size / 2., 0., 0.);
let top_right = Vec3::new(square_size / 2., square_size, 0.);
let top_left = Vec3::new(-square_size / 2., square_size, 0.);

let bottom_left_node = SpawnNode {
    point: Point::new(bottom_left, bottom_left, false),
    connection: Some(vec![top_left, bottom_right]),
    point_material: point_material.clone(),
    connection_material: Some(vec![stick_material.clone(), stick_material.clone()]),
    point_mesh: point_mesh.clone(),
    connection_mesh: Some(vec![stick_mesh.clone(), stick_mesh.clone()]),
    point_size: point_size,
    connection_size: Some(vec![stick_size, stick_size]),
    ..default()
};
let bottom_right_node = SpawnNode {
    point: Point::new(bottom_right, bottom_right + 0.5, false),
    connection: Some(vec![bottom_left, top_right, top_left]),
    point_material: point_material.clone(),
    connection_material: Some(vec![
        stick_material.clone(),
        stick_material.clone(),
        stick_material.clone(),
    ]),
    point_mesh: point_mesh.clone(),
    connection_mesh: Some(vec![
        stick_mesh.clone(),
        stick_mesh.clone(),
        stick_mesh.clone(),
    ]),
    point_size: point_size,
    connection_size: Some(vec![stick_size, stick_size, stick_size]),
    ..default()
};
let top_right_node = SpawnNode {
    point: Point::new(top_right, top_right, false),
    connection: Some(vec![bottom_right, top_left]),
    point_material: point_material.clone(),
    connection_material: Some(vec![stick_material.clone(), stick_material.clone()]),
    point_mesh: point_mesh.clone(),
    connection_mesh: Some(vec![stick_mesh.clone(), stick_mesh.clone()]),
    point_size: point_size,
    connection_size: Some(vec![stick_size, stick_size]),
    ..default()
};
let top_left_node = SpawnNode {
    point: Point::new(top_left, top_left, false),
    connection: Some(vec![bottom_left, top_right, bottom_right]),
    point_material: point_material.clone(),
    connection_material: Some(vec![
        stick_material.clone(),
        stick_material.clone(),
        stick_material.clone(),
    ]),
    point_mesh: point_mesh.clone(),
    connection_mesh: Some(vec![
        stick_mesh.clone(),
        stick_mesh.clone(),
        stick_mesh.clone(),
    ]),
    point_size: point_size,
    connection_size: Some(vec![stick_size, stick_size, stick_size]),
    ..default()
};
let mesh_network = vec![
    bottom_left_node,
    bottom_right_node,
    top_right_node,
    top_left_node,
];

let spawn_custom = {
    let spawn_request = spawn_request.clone();
    let mesh_network = mesh_network.clone();
    move |_| {
        spawn_request
            .send(SpawnRequest::new(mesh_network.clone()))
            .ok();
    }
};
```

## Future Changes

- Ideal gas law: soft bodies with constant (relatively) volumes
- Migration to Leptos 0.8 and Bevy 0.16

## Compatibility

| Crate version | Compatible Leptos version | Compatible Bevy version |
| ------------- | ------------------------- | ----------------------- |
| 1.0           | 0.7                       | 0.15                    |
