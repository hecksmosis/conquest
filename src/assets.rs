use crate::*;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TileAssets::default())
            .add_systems(PreStartup, load_textures);
    }
}

type TextureID = (TileType, usize, Option<Player>);
const TILE_EMPTY: TextureID = (TileType::Empty(Terrain::None), 0, None);

#[derive(Resource, Default)]
pub struct TileAssets {
    textures: HashMap<TextureID, Handle<Image>>,
    pub selector_texture: Handle<Image>,
}

impl TileAssets {
    pub fn get(&self, tile_type: TileType, level: usize, owner: Option<Player>) -> Handle<Image> {
        self.textures
            .get(&(tile_type, level, owner))
            .unwrap_or(self.textures.get(&TILE_EMPTY).unwrap())
            .clone()
    }
}

fn load_textures(asset_server: Res<AssetServer>, mut assets: ResMut<TileAssets>) {
    let mut m = HashMap::new();

    m.insert(TILE_EMPTY, asset_server.load("tile-none.png"));
    m.insert(
        (TileType::Empty(Terrain::Mountain), 0, None),
        asset_server.load("tile-mountain.png"),
    );
    m.insert(
        (TileType::WATER, 0, None),
        asset_server.load("tile-water.png"),
    );

    for player in &[Player::Red, Player::Blue] {
        m.insert(
            (
                TileType::Occupied(PlayerTile::Base, Terrain::None),
                1,
                Some(*player),
            ),
            asset_server.load(format!("base-{}.png", player)),
        );

        m.insert(
            (
                TileType::Occupied(PlayerTile::Tile, Terrain::Mountain),
                1,
                Some(*player),
            ),
            asset_server.load(format!("tile-mountain-{}.png", player)),
        );

        for i in 1..=4 {
            m.insert(
                (
                    TileType::Occupied(PlayerTile::Tile, Terrain::None),
                    i,
                    Some(*player),
                ),
                asset_server.load(format!("tile-{}-{}.png", player, i)),
            );
        }

        for i in 1..=3 {
            m.insert(
                (
                    TileType::Occupied(PlayerTile::Farm, Terrain::None),
                    i,
                    Some(*player),
                ),
                asset_server.load(format!("farm-{}-{}.png", player, i)),
            );
        }
    }

    assets.selector_texture = asset_server.load("selector.png");

    assets.textures = m;
}