# bevy_spacetimedb

Use [SpacetimeDB](https://spacetimedb.com) in your Bevy application.

This plugin will provide you with:

- A resource `StdbConnection` to call your reducers, subscribe to tables, etc.
- Connection lifecycle events: `StdbConnectedEvent`, `StdbDisonnectedEvent`, `StdbConnectionErrorEvent` as Bevy's `EventsReader`
- All the tables events (row inserted/updated/deleted): `InsertEvent\<MyRow>`, `UpdateEvent\<MyRow>`, `DeleteEvent\<MyRow>` as `EventsReader`

This is still WIP and needs a lot of documentation and testing.

## Usage

1. Add the plugin to your bevy application:

```rust
App::new()
    .add_plugins(
        StdbPlugin::default()
            // Required, this method is used to configure your SpacetimeDB connection
            // you will also need to send the connected, disconnected and connect_error with_events back to the plugin
            // Don't forget to call run_threaded() on your connection
            .with_connection(|send_connected, send_disconnected, send_connect_error, _| {
                let conn = DbConnection::builder()
                    .with_module_name("<your module name>")
                    .with_uri("<your spacetimedb instance uri>")
                    .on_connect_error(move |_ctx, err| {
                        send_connect_error
                            .send(StdbConnectionErrorEvent { err })
                            .unwrap();
                    })
                    .on_disconnect(move |_ctx, err| {
                        send_disconnected
                            .send(StdbDisonnectedEvent { err })
                            .unwrap();
                    })
                    .on_connect(move |_ctx, _id, _c| {
                        send_connected.send(StdbConnectedEvent {}).unwrap();
                    })
                    .build()
                    .expect("SpacetimeDB connection failed");

                // Do what you want with the connection here

                // This is very important, otherwise your client will never connect and receive data
                conn.run_threaded();
                conn
            })
            /// Register the events you want to receive (example: players and enemies inserted, updated, deleted) and your reducers
            .with_events(|plugin, app, db| {
                plugin
                    .on_insert(app, db.players())
                    .on_update(app, db.players())
                    .on_delete(app, db.players())
                    .on_insert(app, db.enemies())
                    .on_update(app, db.enemies())
                    .on_delete(app, db.enemies());

                let send_register_player = plugin.reducer_event::<RegisterPlayerEvent>(app);
                reducers.on_register_player(move |ctx, reducer_arg_1, reducer_arg_2| {
                    send_register_player
                        .send(ReducerResultEvent::new(RegisterPlayerEvent {
                            event: ctx.event.clone(),
                            // You can add any data you want here, even reducer arguments
                        }))
                        .unwrap();
                    });
            }),
    );
```

2. Add a system handling connection events
   You can also add systems for `StdbDisonnectedEvent` and `StdbConnectionErrorEvent`

```rust
fn on_connected(
    mut events: EventReader<StdbConnectedEvent>,
    stdb: Res<StdbConnection<DbConnection>>,
) {
    for _ in events.read() {
        info!("Connected to SpacetimeDB");

        // Call any reducers
        stdb.reducers()
            .my_super_reducer("A suuuuppeeeeer argument for a suuuuppeeeeer reducer")
            .unwrap();

        // Subscribe to any tables
        stdb.subscribe()
            .on_applied(|_| info!("Subscription to players applied"))
            .on_error(|_, err| error!("Subscription to players failed for: {}", err))
            .subscribe("SELECT * FROM players");

        // Access your database cache (since it's not yet populated here this line might return 0)
        info!("Players count: {}", stdb.db().players().count());
    }
}
```

3. Add any systems that you need in order to handle the table events you declared and do whatever you want:

```rust
fn on_player_inserted(mut events: EventReader<InsertEvent<Player>>, mut commands: Commands) {
    for event in events.read() {
        commands.spawn(Player { id: event.row.id });
        info!("Player inserted: {:?} -> {:?}", event.row);
    }
}

fn on_player_updated(mut events: EventReader<UpdateEvent<Player>>) {
    for event in events.read() {
        info!("Player updated: {:?} -> {:?}", event.old, event.new);
    }
}

fn on_player_deleted(mut events: EventReader<DeleteEvent<Player>>, q_players: Query<Entity, Player>) {
    for event in events.read() {
        info!("Player deleted: {:?} -> {:?}", event.row);
        // Delete the player's entity
    }
}
```
