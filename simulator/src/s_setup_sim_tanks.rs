use bevy::prelude::*;

use crate::{
    c_client::{Client, DockerClient},
    c_tank::Gun,
    c_tank::Radar,
    c_tank::Tank,
    c_tank::{AllTankInfo, DamageDealer, TankInfo},
    CCollider, CollisionType, Game,
};
use bevy_rapier2d::prelude::*;

use crate::{c_command_source::CommandSource, c_event::EventSink, c_health::Health, CollisionMask};

pub fn create_gun(commands: &mut Commands, x: f32, y: f32) -> Entity {
    let mut t = Transform::from_xyz(x, y, 0.0);
    t.rotate_local_z((Tank::INITIAL_ROTATION).to_radians());
    commands
        .spawn((
            Gun { locked: true },
            SpatialBundle {
                transform: t,
                visibility: Visibility::Visible,
                ..default()
            },
            Sensor,
            GravityScale(0.0),
            RigidBody::Dynamic,
            ColliderMassProperties::Mass(0.0),
            // ColliderMassProperties::Density(1.0),
            Collider::ball(5.0),
            Restitution::coefficient(0.0),
            CollisionGroups::new(
                Group::from_bits_truncate(CollisionMask::NONE),
                Group::from_bits_truncate(CollisionMask::NONE),
            ),
            Damping {
                linear_damping: 0.0,
                angular_damping: 0.0,
            },
            Velocity {
                linvel: Vec2::new(0.0, 0.0),
                angvel: 0.0,
            },
        ))
        .id()
}

pub fn create_radar(commands: &mut Commands, x: f32, y: f32) -> Entity {
    let mut t = Transform::from_xyz(x, y, 0.0);
    t.rotate_local_z((Tank::INITIAL_ROTATION).to_radians());

    commands
        .spawn((
            CCollider {
                collision_type: CollisionType::Radar,
            },
            Radar { locked: true },
            SpatialBundle {
                transform: t,
                visibility: Visibility::Visible,
                ..default()
            },
            Sensor,
            GravityScale(0.0),
            RigidBody::Dynamic,
            ColliderMassProperties::Mass(0.0),
            // ColliderMassProperties::Density(1.0),
            Collider::triangle(
                Vec2::new(0.0, 0.0),
                Vec2::new(-25.0, Game::WIDTH + Game::HEIGHT),
                Vec2::new(25.0, Game::WIDTH + Game::HEIGHT),
            ),
            Restitution::coefficient(0.0),
            CollisionGroups::new(
                Group::from_bits_truncate(CollisionMask::RADAR),
                Group::from_bits_truncate(
                    CollisionMask::TANK | CollisionMask::BULLET | CollisionMask::WALL,
                ),
            ),
            Damping {
                linear_damping: 0.0,
                angular_damping: 0.0,
            },
            Velocity {
                linvel: Vec2::new(0.0, 0.0),
                angvel: 0.0,
            },
        ))
        .id()
}

pub fn create_base_tank(
    tank_info: &TankInfo,
    commands: &mut Commands,
    gun: Entity,
    radar: Entity,
    x: f32,
    y: f32,
    client: impl Component,
) -> Entity {
    let mut t = Transform::from_xyz(x, y, 0.0);
    t.rotate_local_z((Tank::INITIAL_ROTATION).to_radians());
    commands
        .spawn((
            (
                ActiveEvents::COLLISION_EVENTS,
                CCollider {
                    collision_type: CollisionType::Tank,
                },
            ),
            // Sleeping::disabled(),
            // Ccd::enabled(),
            Tank {
                // id: tank_id,
                // hash: tank_hash,
                info: tank_info.clone(),
                cooldown: 0,
                gun,
                radar,
            },
            Health {
                val: Health::MAX_HEALTH,
            },
            DamageDealer { damage_dealt: 0 },
            CommandSource::default(),
            EventSink::default(),
            GravityScale(0.0),
            RigidBody::Dynamic,
            // ColliderMassProperties::Mass(1.0),
            ColliderMassProperties::Density(1.0),
            Collider::ball(Tank::RADIUS),
            (
                Restitution::coefficient(0.0),
                Friction {
                    coefficient: 0.,
                    combine_rule: CoefficientCombineRule::Min,
                },
            ),
            CollisionGroups::new(
                Group::from_bits_truncate(CollisionMask::TANK),
                Group::from_bits_truncate(
                    CollisionMask::TANK
                        | CollisionMask::BULLET
                        | CollisionMask::WALL
                        | CollisionMask::RADAR,
                ),
            ),
            (
                Damping {
                    linear_damping: 0.0,
                    angular_damping: 0.0,
                },
                Velocity {
                    linvel: Vec2::new(0.0, 0.0),
                    angvel: 0.0,
                },
            ),
            client,
            SpatialBundle {
                transform: t,
                visibility: Visibility::Visible,
                ..default()
            },
        ))
        .id()
}

// pub fn create_basic_tank(id: String, i: usize, client: impl Component, commands: &mut Commands) {
//     let x = 150.0 * (i as f32) + 10.0;
//     let y = 0.0;

//     let gun = create_gun(commands, x, y);

//     let radar = create_radar(commands, x, y);

//     create_base_tank(id, i, commands, gun, radar, x, y, client);
// }

pub fn setup_sim_tanks(state: Res<AllTankInfo>, mut commands: Commands) {
    // let game_url = state.all.iter().map(|f| f.hash.to_string()).collect::<Vec<String>>().join("");

    for tank_info in state.all.iter() {
        // create_basic_tank(
        //     tank_id.to_string(),
        //     i,
        //     Client {
        //         client: Box::new(DockerClient {
        //             tank_container_name: tank_container_name.to_string(),
        //         }),
        //     },
        //     &mut commands,
        // );
        let x = 150.0 * (tank_info.index as f32) + 10.0;
        let y = 0.0;

        let gun = create_gun(&mut commands, x, y);
        let radar = create_radar(&mut commands, x, y);

        let client = Client {
            client: Box::new(DockerClient {
                tank_container_name: tank_info.container_name.to_string(),
            }),
        };
        create_base_tank(tank_info, &mut commands, gun, radar, x, y, client);
    }
}
