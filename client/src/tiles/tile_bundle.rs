use crate::*;

#[derive(Bundle, Default)]
pub struct TileBundle {
    position: Position,
    sprite_bundle: SpriteBundle,
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
                texture: assets.get(TileType::EMPTY),
                ..default()
            },
        }
    }
}
