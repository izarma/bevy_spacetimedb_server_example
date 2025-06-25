use bevy::{
    a11y::AccessibilityPlugin, core_pipeline::CorePipelinePlugin, input::InputPlugin, log::LogPlugin, pbr::PbrPlugin, prelude::*, render::{pipelined_rendering::PipelinedRenderingPlugin, RenderPlugin}, scene::ScenePlugin, winit::WinitPlugin
};
use bevy_spacetimedb::{
    DeleteEvent, InsertEvent, ReducerResultEvent, StdbConnectedEvent, StdbConnection,
    StdbConnectionErrorEvent, StdbDisconnectedEvent, StdbPlugin, UpdateEvent,
};
use spacetimedb_sdk::{ReducerEvent, Table};
use stdb::{DbConnection, Entity, EntityTableAccess, Reducer, enter_game};

use crate::stdb::receive_player_input;

mod stdb;

#[derive(Clone, Debug)]
pub struct EnterGameEvent {
    pub event: ReducerEvent<Reducer>,
}

#[derive(Component)]
#[require(Transform)]
pub struct Player {
    id: u32
}

pub fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            LogPlugin::default(),
            InputPlugin::default(),
            AssetPlugin::default(),
            ScenePlugin::default(),
            TransformPlugin::default(),
            RenderPlugin::default(),
            ImagePlugin::default(),
            PipelinedRenderingPlugin::default(),
            CorePipelinePlugin::default(),
            PbrPlugin::default(),
            create_window_plugin(),
            AccessibilityPlugin::default(),
            WinitPlugin::<bevy::winit::WakeUp>::default(),
        ))
        .add_plugins(
            StdbPlugin::default()
                .with_connection(|send_connected, send_disconnected, send_connect_error, _| {
                    let conn = DbConnection::builder()
                        .with_module_name("game1")
                        .with_uri("http://192.168.1.200:5050")
                        .on_connect_error(move |_ctx, err| {
                            send_connect_error
                                .send(StdbConnectionErrorEvent { err })
                                .unwrap();
                        })
                        .on_disconnect(move |_ctx, err| {
                            send_disconnected
                                .send(StdbDisconnectedEvent { err })
                                .unwrap();
                        })
                        .on_connect(move |_ctx, _id, _c| {
                            send_connected.send(StdbConnectedEvent {}).unwrap();
                        })
                        .build()
                        .expect("SpacetimeDB connection failed");

                    conn.run_threaded();
                    conn
                })
                .with_events(|plugin, app, db, reducers| {
                    plugin
                        .on_insert(app, db.entity())
                        .on_update(app, db.entity())
                        .on_delete(app, db.entity());

                    let enter_game = plugin.reducer_event::<EnterGameEvent>(app);
                    reducers.on_enter_game(move |ctx, _name| {
                        enter_game
                            .send(ReducerResultEvent::new(EnterGameEvent {
                                event: ctx.event.clone(),
                            }))
                            .unwrap();
                    });
                }),
        )
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                on_connected,
                on_register_player,
                on_player_inserted,
                on_player_updated,
                on_player_deleted,
                on_keyboard_input,
            ),
        )
        .run();
}

fn create_window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: "spacetest".to_string(),
            ..default()
        }),
        ..default()
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn on_connected(
    mut events: EventReader<StdbConnectedEvent>,
    stdb: Res<StdbConnection<DbConnection>>,
) {
    for _ in events.read() {
        info!("Connected to SpacetimeDB");

        // Call any reducers
        stdb.reducers().enter_game("doodoo".to_owned()).unwrap();
        // Subscribe to any tables
        stdb.subscribe()
            .on_applied(|_| info!("Subscription to players applied"))
            .on_error(|_, err| error!("Subscription to players failed for: {}", err))
            .subscribe("SELECT * FROM entity");

        // Access your database cache (since it's not yet populated here this line might return 0)
        info!("Entity count: {}", stdb.db().entity().count());
    }
}

fn on_register_player(mut events: EventReader<ReducerResultEvent<EnterGameEvent>>) {
    for event in events.read() {
        info!("Entered game: {:?}", event);
    }
}

#[derive(Component)]
pub struct SpacetimeDbEntity {
    pub entity_id: u32,
    pub position: stdb::Vec2,
}

fn on_player_inserted(mut events: EventReader<InsertEvent<Entity>>, mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,) {
    for event in events.read() {
        commands.spawn((SpacetimeDbEntity {
            entity_id: event.row.entity_id,
            position: event.row.position.clone(),
        },
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(event.row.position.x, 0.5, event.row.position.x),
        Player {
            id: event.row.entity_id,
        }
        ));
        info!("Player inserted: {:?}", event.row);
    }
}

fn on_player_updated(mut events: EventReader<UpdateEvent<Entity>>, mut query: Query<(&mut Transform, &Player), With<Player>>) {
    for event in events.read() {
        for (mut transform, player) in query.iter_mut() {
            if event.new.entity_id == player.id {
                transform.translation = Vec3::new(
                    event.new.position.x,
                    0.5,
                    event.new.position.y,
                );
            }
        }
        info!("Player updated: {:?} -> {:?}", event.old, event.new);
    }
}

fn on_player_deleted(mut events: EventReader<DeleteEvent<Entity>>) {
    for event in events.read() {
        info!("Player deleted: {:?}", event.row);
        // Delete the player's entity
    }
}

fn on_keyboard_input(keyboard_input: Res<ButtonInput<KeyCode>>, stdb: Res<StdbConnection<DbConnection>>,) {
    let mut x = 0.0;
    let mut y = 0.0;
    if keyboard_input.pressed(KeyCode::ArrowLeft) { x -= 0.1; }
    if keyboard_input.pressed(KeyCode::ArrowRight) { x += 0.1; }
    if keyboard_input.pressed(KeyCode::ArrowUp) { y += 0.1; }
    if keyboard_input.pressed(KeyCode::ArrowDown) { y -= 0.1; }
    stdb.reducers().receive_player_input(x,y).unwrap();
}