use leptos::prelude::*;
use leptos_verlet::prelude::*;

use crate::components::element_pane::ElementPaneItem;

#[component]
pub fn SpawnIdCard(active_modifier: RwSignal<ModificationTarget>) -> impl IntoView {
    let spawn_request = expect_context::<SpawnSender>();

    let spawn_mesh = RwSignal::new(false);

    const RECT_WIDTH: f32 = 0.75;
    const RECT_HEIGHT: f32 = 1.;

    const POINT_SIZE: f32 = 0.015;
    const STICK_SIZE: f32 = 0.015;

    // Mesh & Materials
    let point_mesh = MeshType::Sphere;
    let stick_mesh = MeshType::Cuboid;
    let rope_material = MaterialType::Color([0., 0., 0., 1.0]);
    let badge_material = MaterialType::Color([1.0, 1.0, 1.0, 0.]);

    // Precompute all the positions
    let rope_top = Vec3::new(0.0, 3.5, 0.0);
    let rope_mid = Vec3::new(0.0, 3.0, 0.0);
    let rope_bottom = Vec3::new(0.0, 2.5, 0.0);
    let half_w = RECT_WIDTH / 2.0;
    let bottom_y = 1.0;
    let top_y = bottom_y + RECT_HEIGHT;
    let center_y = top_y + 0.05;

    let top_left = Vec3::new(-half_w, top_y, 0.0);
    let top_right = Vec3::new(half_w, top_y, 0.0);
    let top_center = Vec3::new(0.0, center_y, 0.0);
    let bottom_left = Vec3::new(-half_w, top_y - RECT_HEIGHT, 0.0);
    let bottom_right = Vec3::new(half_w, top_y - RECT_HEIGHT, 0.0);

    // Spawning a custom mesh when requested
    let centroid: Vec3 = (top_right + bottom_left + bottom_right) / 3.;
    Effect::new(move |_| {
        if spawn_mesh.get() {
            model_loader_with_options(
                "/static/test_card.glb",
                "test_card.glb",
                0,
                Some(Vec3::new(-centroid.y * 3. / 2., 0., 0.)),
                Some(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)),
                Some(0.6),
            );
            spawn_mesh.set(false)
        }
    });

    let make_rope = |pos: Vec3,
                     vel: Vec3,
                     conns: Vec<Vec3>,
                     fixed: bool,
                     extra: f32,
                     attachment: Option<String>| {
        let n = conns.len();
        let custom_material = match rope_material.clone() {
            MaterialType::Color(mut color) => {
                color[3] = 0.0;
                MaterialType::Color(color)
            }
        };
        SpawnNode {
            point: Point::new(pos, vel, fixed),
            connection: Some(conns),
            point_material: custom_material.clone(),
            connection_material: Some(vec![rope_material.clone(); n]),
            point_mesh: point_mesh.clone(),
            connection_mesh: Some(vec![stick_mesh.clone(); n]),
            point_size: POINT_SIZE + extra,
            connection_size: Some(vec![STICK_SIZE + extra; n]),
            attachment,
            point_scale: Vec3::new(0.1, 1., 0.1),
            connection_scale: Some(vec![Vec3::new(1., 1., 0.1); n]),
        }
    };
    let make_badge = |pos: Vec3,
                      vel: Vec3,
                      conns: Vec<Vec3>,
                      fixed: bool,
                      extra: f32,
                      attachment: Option<String>,
                      external_forces: Vec3| {
        let n = conns.len();
        let custom_material = match badge_material.clone() {
            MaterialType::Color(mut color) => {
                color[3] = 0.0;
                MaterialType::Color(color)
            }
        };
        SpawnNode {
            point: Point::new_with_options(pos, vel, fixed, external_forces),
            connection: Some(conns),
            point_material: custom_material.clone(),
            connection_material: Some(vec![badge_material.clone(); n]),
            point_mesh: point_mesh.clone(),
            connection_mesh: Some(vec![stick_mesh.clone(); n]),
            point_size: POINT_SIZE + extra,
            connection_size: Some(vec![STICK_SIZE + extra; n]),
            attachment,
            connection_scale: Some(vec![Vec3::new(1., 1., 1.); n]),
            ..default()
        }
    };

    let mesh_network = vec![
        make_rope(rope_top, rope_top, vec![rope_mid], true, 0.1, None),
        make_rope(
            rope_mid,
            rope_mid,
            vec![rope_top, rope_bottom],
            false,
            0.1,
            None,
        ),
        make_rope(
            rope_bottom,
            rope_bottom,
            vec![rope_mid, top_center],
            false,
            0.1,
            None,
        ),
        make_badge(
            top_center,
            top_center,
            vec![top_left, top_right, rope_bottom],
            false,
            0.0,
            None,
            Vec3::ZERO,
        ),
        make_badge(
            top_right,
            top_right,
            vec![top_center, bottom_right, bottom_left],
            false,
            0.0,
            Some(String::from("test_card.glb")),
            Vec3::new(0.1, 0., 0.),
        ),
        make_badge(
            top_left,
            top_left,
            vec![top_center, bottom_left, bottom_right],
            false,
            0.0,
            None,
            Vec3::new(-0.1, 0., 0.),
        ),
        make_badge(
            bottom_left,
            bottom_left,
            vec![top_right, top_left, bottom_right],
            false,
            0.0,
            Some(String::from("test_card.glb")),
            Vec3::new(-0.1, 0., 0.),
        ),
        make_badge(
            bottom_right,
            bottom_right,
            vec![top_right, top_left, bottom_left],
            false,
            0.0,
            Some(String::from("test_card.glb")),
            Vec3::new(0.1, 0., 0.),
        ),
    ];

    let spawn_custom = {
        let spawn_request = spawn_request.clone();
        let mesh_network = mesh_network.clone();
        move |_| {
            spawn_mesh.set(true);
            let _ = spawn_request.send(SpawnRequest::new(mesh_network.clone()));
        }
    };

    view! {
        <ElementPaneItem
            icon=icondata::FaIdCardClipSolid
            on_click=spawn_custom
            selected_item=active_modifier
            this_item=None
        />
    }
}
