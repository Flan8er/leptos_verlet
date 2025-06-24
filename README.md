[![crates.io](https://img.shields.io/crates/v/leptos_verlet.svg)](https://crates.io/crates/leptos_verlet)
[![docs.rs](https://docs.rs/leptos_verlet/badge.svg)](https://docs.rs/leptos_verlet)

# Leptos Verlet

An engine that allows the addition of interactive verlet simulations into any leptos app.

- Spawned objects are interactive through container bounds allowing for a uniquely interactive component.
- A host of prebuilt objects using an agnostic "builder" layer that will allow, feature is in development, the developer to define and spawn custom objects into the simulation.

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

## Custom Shapes (WIP)

Any shape can be created, simulated, and styled using the built in spawner that reads from a Vec of SpawnNode.

```rust
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
```

Shown below is a verbose use case for spawning a square. As much as feasible, a system should be created for programmatically generating these structures.

The desired vertices of the shape are constructed (below the initial velocity is set to zero by giving the point the same "current position" as "previous position"), and then added to a SpawnNode with the desired connection vertices and mesh/material styling. A mesh_network is then constructed and passed to the built in spawner function (passing the Bevy Commands for spawning).

Note: this process is not finalized and is subject to change in the next few releases, especially the dependence on Bevy resources inside the Leptos app.

```rust
let square_size = 0.45;
let stick_mesh = meshes.add(Cuboid::default());
let point_mesh = meshes.add(Sphere::default());

let bottom_left = Vec3::new(
    position[0] - square_size,
    position[1] - square_size,
    position[2],
);
let bottom_right = Vec3::new(
    position[0] + square_size,
    position[1] - square_size,
    position[2],
);
let top_right = Vec3::new(
    position[0] + square_size,
    position[1] + square_size,
    position[2],
);
let top_left = Vec3::new(
    position[0] - square_size,
    position[1] + square_size,
    position[2],
);

let bottom_left_node = SpawnNode {
    point: Point::new(bottom_left, bottom_left, false),
    connection: Some(vec![top_left, bottom_right]),
    point_material: point_material.clone(),
    connection_material: Some(vec![stick_material.clone(), stick_material.clone()]),
    point_mesh: point_mesh.clone(),
    connection_mesh: Some(vec![stick_mesh.clone(), stick_mesh.clone()]),
    point_size: POINT_SIZE,
    connection_size: Some(vec![STICK_SIZE, STICK_SIZE]),
};
let bottom_right_node = SpawnNode {
    point: Point::new(bottom_right, bottom_right, false),
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
    point_size: POINT_SIZE,
    connection_size: Some(vec![STICK_SIZE, STICK_SIZE, STICK_SIZE]),
};
let top_right_node = SpawnNode {
    point: Point::new(top_right, top_right, false),
    connection: Some(vec![bottom_right, top_left]),
    point_material: point_material.clone(),
    connection_material: Some(vec![stick_material.clone(), stick_material.clone()]),
    point_mesh: point_mesh.clone(),
    connection_mesh: Some(vec![stick_mesh.clone(), stick_mesh.clone()]),
    point_size: POINT_SIZE,
    connection_size: Some(vec![STICK_SIZE, STICK_SIZE]),
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
    point_size: POINT_SIZE,
    connection_size: Some(vec![STICK_SIZE, STICK_SIZE, STICK_SIZE]),
};
let mesh_network = vec![
    bottom_left_node,
    bottom_right_node,
    top_right_node,
    top_left_node,
];

spawner(mesh_network, commands);
```

## Compatibility

| Crate version | Compatible Leptos version | Compatible Bevy version |
| ------------- | ------------------------- | ----------------------- |
| 1.0           | 0.7                       | 0.15                    |
