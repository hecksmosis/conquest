use crate::*;

#[derive(Bundle, Default)]
pub struct TileBundle {
    position: Position,
    sprite_bundle: SpriteBundle,
    tile: Tile,
    level: Level,
    owned: Owned,
    health: Health,
}

impl TileBundle {
    pub fn blank(Vec2 { x, y }: Vec2, assets: &TileAssets) -> Self {
        let position =
            Position(Vec2::new(x - MAP_WIDTH, y - MAP_HEIGHT) * TILE_SIZE + TILE_SIZE / 2.0);
        let transform = position.clone().into();

        Self {
            position,
            sprite_bundle: SpriteBundle {
                transform,
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(TILE_SIZE)),
                    ..default()
                },
                texture: assets.get(TileType::EMPTY, 0, None),
                ..default()
            },
            ..default()
        }
    }

    pub fn base(player: Player, assets: &TileAssets) -> Self {
        let position = match &player {
            Player::Red => {
                Position(Vec2::new(-MAP_WIDTH, -MAP_HEIGHT) * TILE_SIZE + TILE_SIZE / 2.0)
            }
            Player::Blue => {
                Position(Vec2::new(MAP_WIDTH - 1.0, MAP_HEIGHT - 1.0) * TILE_SIZE + TILE_SIZE / 2.0)
            }
        };
        let transform = position.clone().into();

        Self {
            position,
            sprite_bundle: SpriteBundle {
                transform,
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(TILE_SIZE)),
                    ..default()
                },
                texture: assets.get(
                    TileType::Occupied(PlayerTile::Base, Terrain::None),
                    1,
                    Some(player),
                ),
                ..default()
            },
            tile: Tile(TileType::Occupied(PlayerTile::Base, Terrain::None)),
            owned: Owned(Some(player)),
            health: Health(2),
            level: Level(1),
        }
    }
}
