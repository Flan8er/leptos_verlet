use bevy::{
    asset::io::{
        AssetSource, AssetSourceId,
        memory::{Dir, MemoryAssetReader},
    },
    prelude::*,
};
use leptos::{prelude::*, server_fn::request::browser::Request, task::spawn_local};
use leptos_bevy_canvas::prelude::*;
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    path::Path,
};

use crate::plugins::attachment::plugin::AttachmentPoint;

#[derive(Resource)]
struct MemoryDir {
    dir: Dir,
}

#[derive(Event)]
pub struct LoadModelEvent {
    pub name: String,
    pub bytes: Vec<u8>,
    pub scene_index: usize,
}

pub struct AssetLoaderPlugin;
impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        let memory_dir = MemoryDir {
            dir: Dir::default(),
        };
        let reader = MemoryAssetReader {
            root: memory_dir.dir.clone(),
        };

        app.register_asset_source(
            AssetSourceId::from_static("memory"),
            AssetSource::build().with_reader(move || Box::new(reader.clone())),
        )
        .insert_resource(memory_dir)
        .add_systems(PreUpdate, load_model_system);
    }
}

fn load_model_system(
    mut events: EventReader<LoadModelEvent>,
    dir: ResMut<MemoryDir>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for LoadModelEvent {
        name,
        bytes,
        scene_index,
    } in events.read()
    {
        // insert the bytes under the given filename
        dir.dir.insert_asset(Path::new(name), bytes.clone());

        // and then load + spawn it immediately
        let uri = format!("memory://{}#Scene{}", name, scene_index);
        let handle: Handle<Scene> = asset_server.load(&uri);

        // Hash the file name
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);

        commands.spawn((
            SceneRoot(handle),
            Transform::from_xyz(0.0, 0.0, 0.0),
            Visibility::default(),
            AttachmentPoint(hasher.finish()),
        ));
    }
}

async fn fetch_and_send_model(
    url: String,
    model_name: String,
    scene_index: usize,
    sender: LeptosEventSender<LoadModelEvent>,
) {
    let resp = match Request::get(&url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            leptos::logging::log!("Err: Url request failed. {}", e);
            return;
        }
    };

    let bytes = match resp.binary().await {
        Ok(bytes) => bytes,
        Err(e) => {
            leptos::logging::log!("Err: Failed to parse bytes from response. {}", e);
            return;
        }
    };

    sender
        .send(LoadModelEvent {
            name: model_name,
            bytes,
            scene_index,
        })
        .ok();
}

/// NOTE: This function must be placed in a reactive context.
///
/// This function take in the relative path to the desired asset and spawns it into the bevy world.
///
/// Standard bevy syntax is:
///
/// asset_server: Res<AssetServer>
///
/// let my_gltf = asset_server.load("example.glb#Scene0");
///
/// Conforming to bevy conventions the inputs are similar:
///
/// asset_path: The path, relative to your Leptos app, to the desired asset.
/// Ex. For a file in your public folder: "/example.glb"
///
/// model_name: The name of the file. Consistency here is key as this file name in combination with the "attachment" field on a spawned Point
/// will track positioning and orientation of the model within the simulation.
///  Ex. "example.glb"
///
/// scene_index: The desired sub-asset from the asset file. Ex. "0", "1", ... for "#Scene0", "#Scene1", ...
pub fn model_loader<T>(asset_path: T, model_name: T, scene_index: usize)
where
    T: Into<String>,
{
    let asset_path: String = asset_path.into();
    let model_name: String = model_name.into();

    // let ctx = expect_context::<MyContext>(cx);
    let event_sender = expect_context::<LeptosEventSender<LoadModelEvent>>();

    spawn_local(fetch_and_send_model(
        asset_path,
        model_name,
        scene_index,
        event_sender,
    ));
}
