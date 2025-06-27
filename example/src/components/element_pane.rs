use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_verlet::prelude::*;

#[component]
pub fn ElementPane(active_modifier: RwSignal<ModificationTarget>) -> impl IntoView {
    let target_sender = expect_context::<ModificationTargetSender>();
    let spawn_monkey = RwSignal::new(false);

    // Spawning a custom mesh when requested
    Effect::new(move |_| {
        if spawn_monkey.get() {
            model_loader("/static/monkey.glb", "monkey.glb", 0);
            spawn_monkey.set(false)
        }
    });

    let modification_point = {
        let sender = target_sender.clone();
        move |_| {
            sender.send(ModificationTarget::Point).ok();
            active_modifier.set(ModificationTarget::Point)
        }
    };
    let modification_line = {
        let sender = target_sender.clone();
        move |_| {
            sender.send(ModificationTarget::Line).ok();
            active_modifier.set(ModificationTarget::Line)
        }
    };
    let modification_lock = {
        let sender = target_sender.clone();
        move |_| {
            sender.send(ModificationTarget::Lock).ok();
            active_modifier.set(ModificationTarget::Lock)
        }
    };
    let modification_cut = {
        let sender = target_sender.clone();
        move |_| {
            sender.send(ModificationTarget::Cut).ok();
            active_modifier.set(ModificationTarget::Cut)
        }
    };
    let modification_none = {
        let sender = target_sender.clone();
        move |_| {
            sender.send(ModificationTarget::None).ok();
            active_modifier.set(ModificationTarget::None)
        }
    };

    let spawn_square = {
        let sender = target_sender.clone();
        move |_| {
            sender.send(ModificationTarget::SpawnSquare).ok();
            active_modifier.set(ModificationTarget::SpawnSquare)
        }
    };
    let spawn_rope = {
        let sender = target_sender.clone();
        move |_| {
            sender.send(ModificationTarget::SpawnRope).ok();
            active_modifier.set(ModificationTarget::SpawnRope)
        }
    };
    let spawn_cloth = {
        let sender = target_sender.clone();
        move |_| {
            sender.send(ModificationTarget::SpawnCloth).ok();
            active_modifier.set(ModificationTarget::SpawnCloth)
        }
    };
    let spawn_cube = {
        let sender = target_sender.clone();
        move |_| {
            sender.send(ModificationTarget::SpawnCube).ok();
            active_modifier.set(ModificationTarget::SpawnCube)
        }
    };
    let point_info = {
        let sender = target_sender.clone();
        move |_| {
            sender.send(ModificationTarget::PointInfo).ok();
            active_modifier.set(ModificationTarget::PointInfo)
        }
    };
    let set_spawn_monkey = move |_| spawn_monkey.update(|c| *c = !*c);

    view! {
        <div class="absolute top-2 left-[1/2] -translate-x-[1/2] p-1 border flex items-center justify-between rounded-md w-[calc(50dvw)] glass z-[20]">
            <div class="flex gap-4">
                <ElementPaneItem icon=icondata::CgEditBlackPoint on_click=modification_point selected_item=active_modifier this_item=Some(ModificationTarget::Point)/>
                <ElementPaneItem icon=icondata::BsLink45deg on_click=modification_line selected_item=active_modifier this_item=Some(ModificationTarget::Line)/>
                <ElementPaneItem icon=icondata::LuSquareSlash on_click=spawn_square selected_item=active_modifier this_item=Some(ModificationTarget::SpawnSquare)/>
                <ElementPaneItem icon=icondata::TbCube on_click=spawn_cube selected_item=active_modifier this_item=Some(ModificationTarget::SpawnCube)/>
                <ElementPaneItem icon=icondata::MdiTransitConnection on_click=spawn_rope selected_item=active_modifier this_item=Some(ModificationTarget::SpawnRope)/>
                <ElementPaneItem icon=icondata::CgMenuGridO on_click=spawn_cloth selected_item=active_modifier this_item=Some(ModificationTarget::SpawnCloth)/>
                <LeptosSpawner active_modifier/>
                <ElementPaneItem icon=icondata::LuImport on_click=set_spawn_monkey selected_item=active_modifier this_item=None/>
            </div>

            <div class="flex gap-4">
                <ElementPaneItem icon=icondata::AiInfoCircleOutlined on_click=point_info selected_item=active_modifier this_item=Some(ModificationTarget::PointInfo)/>
                <ElementPaneItem icon=icondata::BiLockOpenAltRegular on_click=modification_lock selected_item=active_modifier this_item=Some(ModificationTarget::Lock)/>
                <ElementPaneItem icon=icondata::IoCut on_click=modification_cut selected_item=active_modifier this_item=Some(ModificationTarget::Cut)/>
                <ElementPaneItem icon=icondata::IoClose on_click=modification_none selected_item=active_modifier this_item=Some(ModificationTarget::None)/>
            </div>
        </div>
    }
}

#[component]
fn ElementPaneItem<F>(
    icon: icondata::Icon,
    on_click: F,
    selected_item: RwSignal<ModificationTarget>,
    this_item: Option<ModificationTarget>,
) -> impl IntoView
where
    F: Fn(web_sys::MouseEvent) + Clone + 'static,
{
    view! {
        <div
            class=move || {
                format!("w-[35px] h-[35px] rounded-md hover:bg-card-active p-[4px] cursor-grab duration-200 {}",
                    if let Some(this_item) = this_item.clone() {
                        if selected_item.get() == this_item {
                            "bg-card-active"
                        } else {
                            "bg-none"
                        }
                    } else {
                        "bg-none"
                    }
                )
            }
            on:click=on_click
        >
            <Icon icon width="100%" height="100%"/>
        </div>
    }
}

#[component]
fn LeptosSpawner(active_modifier: RwSignal<ModificationTarget>) -> impl IntoView {
    let spawn_request = expect_context::<SpawnSender>();

    let square_size = 0.45;
    let point_size = 0.025;
    let stick_size = 0.01;
    let point_mesh = MeshType::Sphere;
    let stick_mesh = MeshType::Cuboid;
    let point_material = MaterialType::Color([1., 1., 1., 1.]);
    let stick_material = MaterialType::Color([1., 1., 1., 0.75]);

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
    view! {
        <ElementPaneItem icon=icondata::FiEdit on_click=spawn_custom selected_item=active_modifier this_item=None/>
    }
}
