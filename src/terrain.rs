use crate::{hud::PlacementModeText, *};

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TerrainCounter::default())
            .add_systems(OnEnter(GameState::Terrain), reset)
            .add_systems(
                Update,
                (
                    set_placement_mode,
                    make_terrain,
                    finish_placement,
                    set_placement_mode_text,
                )
                    .run_if(in_state(GameState::Terrain)),
            )
            .add_systems(OnExit(GameState::Terrain), remove_placement_text);
    }
}

#[derive(Resource, Clone, Debug)]
struct TerrainCounter {
    pub placement_mode: Terrain,
    pub mountain_count: [usize; 2],
    pub water_count: [usize; 2],
}

impl Default for TerrainCounter {
    fn default() -> Self {
        Self {
            placement_mode: Terrain::Mountain,
            mountain_count: [0, 0],
            water_count: [0, 0],
        }
    }
}

impl TerrainCounter {
    pub fn add(&mut self, tile_type: Terrain, player: Player) -> Result<(), Box<dyn Error>> {
        if (tile_type == Terrain::Mountain
            && self.mountain_count[player as usize] >= MAX_MOUNTAIN_COUNT)
            || (tile_type == Terrain::Water && self.water_count[player as usize] >= MAX_WATER_COUNT)
        {
            return Err("Too many mountains or water".into());
        }

        match tile_type {
            Terrain::Mountain => self.mountain_count[player as usize] += 1,
            Terrain::Water => self.water_count[player as usize] += 1,
            _ => (),
        }

        Ok(())
    }

    pub fn remove(&mut self, tile_type: Terrain, player: Player) {
        match tile_type {
            Terrain::Mountain => self.mountain_count[player as usize] -= 1,
            Terrain::Water => self.water_count[player as usize] -= 1,
            _ => (),
        }
    }
}

fn set_placement_mode(keys: Res<Input<KeyCode>>, mut terrain_counter: ResMut<TerrainCounter>) {
    if keys.just_pressed(KeyCode::M) {
        terrain_counter.placement_mode = Terrain::Mountain;
    } else if keys.just_pressed(KeyCode::W) {
        terrain_counter.placement_mode = Terrain::Water;
    }
}

fn finish_placement(
    keys: Res<Input<KeyCode>>,
    turn: Res<TurnCounter>,
    mut state: ResMut<NextState<GameState>>,
    mut terrain_counter: ResMut<TerrainCounter>,
    mut turn_event: EventWriter<TurnEvent>,
) {
    if keys.just_pressed(KeyCode::Return) {
        if turn.player() == Player::Blue {
            state.set(GameState::Game);
        }
        terrain_counter.placement_mode = Terrain::Mountain;
        turn_event.send(TurnEvent);
    }
}

fn make_terrain(
    assets: Res<TileAssets>,
    (mouse, buttons): (Res<GridMouse>, Res<Input<MouseButton>>),
    mut tile_query: Query<(
        &Position,
        &mut Tile,
        &Owned,
        &mut Health,
        &mut Handle<Image>,
    )>,
    turn: Res<TurnCounter>,
    mut terrain_counter: ResMut<TerrainCounter>,
) {
    if !buttons.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
        return;
    }

    let button = buttons.get_just_pressed().next().unwrap();
    if let Some((_, mut tile, Owned(None), mut health, mut image)) =
        tile_query.iter_mut().find(|(pos, tile, ..)| {
            pos == &&mouse.as_position()
                && match button {
                    MouseButton::Left => true,
                    MouseButton::Right => tile.0.is_empty(),
                    _ => unreachable!(),
                }
        })
    {
        match (button, terrain_counter.placement_mode) {
            (MouseButton::Right, terrain @ (Terrain::Mountain | Terrain::Water)) => {
                if terrain_counter.add(terrain, turn.player()).is_ok() {
                    tile.0 = TileType::Empty(terrain);
                    if terrain == Terrain::Mountain {
                        health.0 = 2;
                    }
                }
            }
            (MouseButton::Left, _) => {
                terrain_counter.remove(tile.0.terrain(), turn.player());
                tile.0 = TileType::EMPTY;
            }
            _ => (),
        };
        *image = assets.get(tile.0, 0, None);
    }
}

fn set_placement_mode_text(
    mut query: Query<&mut Text, With<PlacementModeText>>,
    mode: Res<TerrainCounter>,
) {
    for mut text in query.iter_mut() {
        text.sections[1].value = match mode.placement_mode {
            Terrain::Mountain => "Mountain".to_string(),
            Terrain::Water => "Water".to_string(),
            _ => "None".to_string(),
        };
    }
}

fn remove_placement_text(mut commands: Commands, query: Query<Entity, With<PlacementModeText>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn reset(mut terrain_counter: ResMut<TerrainCounter>) {
    *terrain_counter = default()
}
