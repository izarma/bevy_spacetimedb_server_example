use bevy::{log::LogPlugin, prelude::*};
use bevy_spacetimedb::{
    ReducerResultEvent, StdbConnectedEvent, StdbConnection, StdbConnectionErrorEvent,
    StdbDisconnectedEvent, StdbPlugin, InsertEvent, UpdateEvent, DeleteEvent
};
use spacetimedb_sdk::{ReducerEvent, Table};
use stdb::{enter_game, DbConnection, Entity, EntityTableAccess, Reducer};

mod stdb;

#[derive(Clone, Debug)]
pub struct EnterGameEvent {
    pub event: ReducerEvent<Reducer>,
}

pub fn main() {
    App::new()
        .add_plugins((MinimalPlugins, LogPlugin::default()))
        .add_plugins(
            StdbPlugin::default()
                .with_connection(|send_connected, send_disconnected, send_connect_error, _| {
                    let conn = DbConnection::builder()
                        .with_module_name("game1")
                        .with_uri("http://127.0.0.1:3000")
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
        .add_systems(Startup, hello_world)
        .add_systems(Update, (
            on_connected, 
            on_register_player,
            on_player_inserted,
            on_player_updated,
            on_player_deleted
        ))
        .run();
}

fn hello_world() {
    info!("Hello, world!");
}

fn on_connected(
    mut events: EventReader<StdbConnectedEvent>,
    stdb: Res<StdbConnection<DbConnection>>,
) {
    for _ in events.read() {
        info!("Connected to SpacetimeDB");

        // Call any reducers
        stdb.reducers().enter_game("kon".to_owned()).unwrap();
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

fn on_player_inserted(mut events: EventReader<InsertEvent<Entity>>, mut commands: Commands) {
    for event in events.read(){
        commands.spawn(SpacetimeDbEntity { 
            entity_id: event.row.entity_id,
            position: event.row.position.clone(),
        });
        info!("Player inserted: {:?}", event.row);
    }
}

fn on_player_updated(mut events: EventReader<UpdateEvent<Entity>>) {
    for event in events.read() {
        info!("Player updated: {:?} -> {:?}", event.old, event.new);
    }
}

fn on_player_deleted(mut events: EventReader<DeleteEvent<Entity>>) {
    for event in events.read() {
        info!("Player deleted: {:?}", event.row);
        // Delete the player's entity
    }
}