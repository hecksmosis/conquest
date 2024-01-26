use crate::*;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TileAssets::default())
            .add_systems(PreStartup, load_textures);
    }
}

#[derive(Resource, Default)]
pub struct TileAssets {
    textures: HashMap<TileType, Handle<Image>>,
    pub selector_texture: Handle<Image>,
}

impl TileAssets {
    pub fn get(&self, tile_type: TileType) -> Handle<Image> {
        self.textures
            .get(&tile_type)
            .unwrap_or(self.textures.get(&TileType::EMPTY).unwrap())
            .clone()
    }
}

fn load_textures(asset_server: Res<AssetServer>, mut assets: ResMut<TileAssets>) {
    let mut m = HashMap::new();

    m.insert(TileType::EMPTY, asset_server.load("tile-none.png"));
    m.insert(TileType::WATER, asset_server.load("tile-water.png"));
    m.insert(
        TileType::Empty(Terrain::Mountain),
        asset_server.load("tile-mountain.png"),
    );

    // TODO: textures for damaged tiles
    for player in &[Player::Red, Player::Blue] {
        m.insert(
            TileType::Occupied {
                player_tile: PlayerTile::Base,
                terrain: Terrain::None,
                owner: *player,
                level: 1,
                hp: 2,
            },
            asset_server.load(format!("base-{}.png", player)),
        );

        m.insert(
            TileType::Occupied {
                player_tile: PlayerTile::Tile,
                terrain: Terrain::Mountain,
                owner: *player,
                level: 1,
                hp: 2,
            },
            asset_server.load(format!("tile-mountain-{}.png", player)),
        );

        for i in 1..=4 {
            m.insert(
                TileType::Occupied {
                    player_tile: PlayerTile::Tile,
                    terrain: Terrain::None,
                    owner: *player,
                    level: i,
                    hp: i,
                },
                asset_server.load(format!("tile-{}-{}.png", player, i)),
            );
        }

        for i in 1..=3 {
            m.insert(
                TileType::Occupied {
                    player_tile: PlayerTile::Farm,
                    terrain: Terrain::None,
                    owner: *player,
                    level: i,
                    hp: i,
                },
                asset_server.load(format!("farm-{}-{}.png", player, i)),
            );
        }
    }

    assets.selector_texture = asset_server.load("selector.png");

    assets.textures = m;
}
