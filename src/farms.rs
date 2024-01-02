use crate::{utils::count_farms, *};

#[derive(Resource, Clone, Debug, Default)]
pub struct FarmCounter {
    pub counts: [usize; 2],
    pub points: [usize; 2],
}

impl FarmCounter {
    pub fn available_farms(&self, player: Player) -> usize {
        self.counts[player as usize] - self.points[player as usize]
    }
}

pub struct FarmPlugin;

impl Plugin for FarmPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FarmCounter::default())
            .add_systems(Update, (farm_counter, point_counter));
    }
}

fn farm_counter(farms: Query<(&Tile, &Owned, &Level)>, mut counter: ResMut<FarmCounter>) {
    counter.counts = count_farms(&farms);
}

fn point_counter(tiles: Query<(&Tile, &Owned, &Level)>, mut counter: ResMut<FarmCounter>) {
    counter.points = tiles.iter().filter(|(Tile(tile), ..)| tile.is_tile()).fold(
        [0; 2],
        |mut total, (_, Owned(owner), Level(level))| {
            let Some(player) = owner else {
                return total;
            };
            total[*player as usize] += level;
            total
        },
    )
}
