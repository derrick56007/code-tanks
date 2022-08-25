use std::process::Command;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;
use ctsimlib::{
    s_apply_commands::apply_commands,
    s_physics::{physics, physics2},
    s_request_commands::request_commands,
    s_request_commands_by_event::request_commands_by_event,
    s_walls::setup_walls,
    *,
};
use s_graphics::setup_graphics;
use s_request_debug_commands::request_debug_commands;
use s_setup_tanks::setup_tanks;
use s_update_health::update_health;

pub mod s_graphics;
pub mod s_request_debug_commands;
pub mod s_setup_tanks;
pub mod s_update_health;

pub fn run_tank(url: &str, game_url: &str, post_fix: usize) -> String {
    // docker run -d --network=codetanks_default -p  8080:8080 --name tank_id --label com.docker.compose.project=codetanks localhost:5001/url
    let tank_id = format!("{}-{}-{}", game_url, url, post_fix);
    let output_raw = Command::new("docker")
        .arg("run")
        .arg("-d")
        .arg("--network=codetanks_default")
        .arg("-p")
        .arg("8080")
        .arg("--name")
        .arg(&tank_id)
        .arg("--label")
        .arg("com.docker.compose.project=codetanks")
        .arg(format!("localhost:5001/{}", url))
        .output()
        .expect("failed to communicate with docker");
    let result_raw = String::from_utf8_lossy(&output_raw.stdout);
    // let err_raw = String::from_utf8_lossy(&output_raw.stderr);

    println!("run stdout:");
    println!("{}", result_raw.to_string());

    // let output_raw = Command::new("bash")
    //     .arg("-c")
    //     .arg(format!(r#"docker port {} 8080 | cut -d: -f2"#, tank_id))
    //     .output()
    //     .expect("failed to communicate with docker");
    // let result_raw = String::from_utf8_lossy(&output_raw.stdout);
    // // let err_raw = String::from_utf8_lossy(&output_raw.stderr);
    // let port = result_raw
    //     .to_string()
    //     .split("\n")
    //     .map(|f| f.to_string())
    //     .collect::<Vec<String>>();

    // let port = port.first().unwrap();

    // println!("port stdout:");
    // println!("{}", port);
    // format!("{}:{}", tank_id, port)
    tank_id
}

pub fn run_game(args: &[String]) {
    let game_url = args.join("");
    let tank_ids = args
        .iter()
        .enumerate()
        .map(|(i, url)| run_tank(url, &game_url, i))
        .collect::<Vec<String>>();

    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        // .init_resource::<CState>()
        .insert_resource(CState {
            tick: 0,
            tank_ids: tank_ids.to_vec(),
        })
        // .add_asset::<CustomAsset>()
        // .init_asset_loader::<CustomAssetLoader>()
        // .add_startup_system(load_tanks)
        .add_system(setup_tanks)
        // .add_system(print_on_load)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup_graphics)
        .add_startup_system(setup_walls)
        .add_stage(
            "request_commands",
            SystemStage::single_threaded().with_system(request_commands),
        )
        .add_stage(
            "request_debug_commands",
            SystemStage::single_threaded().with_system(request_debug_commands),
        )
        .add_stage(
            "apply_commands",
            SystemStage::single_threaded().with_system(apply_commands),
        )
        .add_stage(
            "physics2",
            SystemStage::single_threaded().with_system(physics2),
        )
        .add_stage(
            "physics",
            SystemStage::single_threaded().with_system(physics),
        )
        .add_stage(
            "publish_events",
            SystemStage::single_threaded().with_system(request_commands_by_event),
        )
        .add_stage(
            "update_health",
            SystemStage::single_threaded().with_system(update_health),
        )
        .run();
}
