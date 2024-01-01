use crate::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                debug_tile_adyacents,
                debug_base,
                list_entities,
                debug_connected_to_base,
            ),
        );
    }
}

fn debug_base(
    tile_query: Query<(&Position, &Tile, &Level, &Owned, &Health)>,
    keys: Res<Input<KeyCode>>,
    grid: Res<TileGrid>,
) {
    if !keys.just_pressed(KeyCode::B) {
        return;
    }

    info!("Debugging base");

    for (pos, tile, level, owned, health) in
        tile_query.iter().filter(|(_, tile, ..)| tile.0.is_base())
    {
        println!(
            "Position: {:?}, Tile: {:?}, Level: {}, Owned: {:?}, Health: {}, Grid tile: {:?}",
            pos.0,
            tile,
            level.0,
            owned,
            health.0,
            grid.get_tile(pos.clone().as_grid_index())
        );
    }
}

#[allow(clippy::type_complexity)]
fn list_entities(
    keys: Res<Input<KeyCode>>,
    query: Query<(
        Entity,
        &Transform,
        Option<&Tile>,
        Option<&Level>,
        Option<&Owned>,
        Option<&Health>,
        Option<&Text>,
    )>,
) {
    if !keys.just_pressed(KeyCode::L) {
        return;
    }

    info!("Listing entities");

    for (entity, pos, tile, level, owned, health, text) in query.iter() {
        println!(
            "Entity: {:?}, Position: {:?}, Tile: {:?}, Level: {:?}, Owned: {:?}, Health: {:?}, Text: {:?}",
            entity, pos, tile, level, owned, health, text
        );
    }
}

fn debug_tile_adyacents(
    mouse: Res<GridMouse>,
    tile_query: Query<(&Position, &Tile, &Level, &Owned, &Health)>,
    keys: Res<Input<KeyCode>>,
    turn: Res<TurnCounter>,
    grid: Res<TileGrid>,
) {
    if !keys.just_pressed(KeyCode::D) {
        return;
    }

    info!("Debugging tile at {:?}", mouse.as_position());

    if let Some((pos, tile, level, owned, health)) = tile_query
        .iter()
        .find(|(pos, ..)| pos == &&mouse.as_position())
    {
        println!(
            "Position: {:?}, Tile: {:?}, Level: {}, Owned: {:?}, Health: {}, Grid tile: {:?}, Grid tile adyacents: {:?}",
            pos.0,
            tile,
            level.0,
            owned,
            health.0,
            grid.get_tile(pos.clone().as_grid_index()),
            grid.get_connected_tiles(pos.clone().as_grid_index(), turn.player())
        );
    }
}

fn debug_connected_to_base(
    mouse: Res<GridMouse>,
    tile_query: Query<(&Position, &Tile, &Level, &Owned, &Health)>,
    keys: Res<Input<KeyCode>>,
    turn: Res<TurnCounter>,
    grid: Res<TileGrid>,
) {
    if !keys.just_pressed(KeyCode::C) {
        return;
    }

    info!("Debugging tile at {:?}", mouse.as_position());

    if let Some((pos, tile, level, owned, health)) = tile_query
        .iter()
        .find(|(pos, ..)| pos == &&mouse.as_position())
    {
        println!(
            "Position: {:?}, Tile: {:?}, Level: {}, Owned: {:?}, Health: {}, Grid tile: {:?}, Connected to base: {}",
            pos.0,
            tile,
            level.0,
            owned,
            health.0,
            grid.get_tile(pos.clone().as_grid_index()),
            grid.is_connected_to_base(pos.clone().as_grid_index(), turn.player())
        );
    }
}
