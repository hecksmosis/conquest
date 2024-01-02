use super::*;

#[test]
fn test_in_bounds() {
    assert!(TileGrid::in_bounds(0.0, 0.0));
    assert!(!TileGrid::in_bounds(-MAP_WIDTH - 1.0, 0.0));
    assert!(!TileGrid::in_bounds(0.0, -MAP_HEIGHT - 1.0));
    assert!(!TileGrid::in_bounds(MAP_WIDTH, 0.0));
    assert!(!TileGrid::in_bounds(0.0, MAP_HEIGHT));
}

#[test]
fn test_get_index() {
    assert_eq!(TileGrid::get_index(Vec2 { x: 0.0, y: 0.0 }), 72);
}

#[test]
fn test_get_tile() {
    let mut grid = TileGrid {
        grid: [(None, false); (MAP_WIDTH * MAP_HEIGHT * 4.0 + 1.0) as usize],
    };
    grid.grid[0] = (Some(Player::Red), true);
    assert_eq!(
        grid.get_tile(Vec2 {
            x: -MAP_WIDTH,
            y: -MAP_HEIGHT
        }),
        (Some(Player::Red), true)
    );
}

#[test]
fn test_set_owner() {
    let mut grid = TileGrid {
        grid: [(None, false); (MAP_WIDTH * MAP_HEIGHT * 4.0 + 1.0) as usize],
    };
    grid.set_owner(Vec2 { x: 0.0, y: 0.0 }, Some(Player::Red));
    assert_eq!(
        grid.get_tile(Vec2 { x: 0.0, y: 0.0 }),
        (Some(Player::Red), false)
    );
}

#[test]
fn test_make_base() {
    let mut grid = TileGrid {
        grid: [(None, false); (MAP_WIDTH * MAP_HEIGHT * 4.0 + 1.0) as usize],
    };
    grid.make_base(Player::Red);
    assert_eq!(
        grid.get_tile(Vec2::new(-MAP_WIDTH, -MAP_HEIGHT)),
        (Some(Player::Red), true)
    );
}

#[test]
fn test_get_connected_tiles() {
    let mut grid = TileGrid {
        grid: [(None, false); (MAP_WIDTH * MAP_HEIGHT * 4.0 + 1.0) as usize],
    };
    grid.set_owner(Vec2 { x: 0.0, y: 0.0 }, Some(Player::Red));
    grid.set_owner(Vec2 { x: 1.0, y: 0.0 }, Some(Player::Red));
    let connected_tiles = grid.get_connected_tiles(Vec2 { x: 0.0, y: 0.0 }, Player::Red);
    assert_eq!(connected_tiles.len(), 1);
    assert_eq!(connected_tiles[0], Vec2 { x: 1.0, y: 0.0 });
}

#[test]
fn test_is_connected_to_base() {
    let mut grid = TileGrid {
        grid: [(None, false); (MAP_WIDTH * MAP_HEIGHT * 4.0 + 1.0) as usize],
    };
    grid.set_owner(
        Vec2 {
            x: -MAP_WIDTH + 1.0,
            y: -MAP_HEIGHT,
        },
        Some(Player::Red),
    );
    grid.set_owner(
        Vec2 {
            x: -MAP_WIDTH + 1.0,
            y: -MAP_HEIGHT + 1.0,
        },
        Some(Player::Red),
    );
    grid.set_owner(
        Vec2 {
            x: -MAP_WIDTH + 2.0,
            y: -MAP_HEIGHT,
        },
        Some(Player::Red),
    );
    grid.make_base(Player::Red);
    assert!(grid.is_connected_to_base(
        Vec2 {
            x: -MAP_WIDTH + 2.0,
            y: -MAP_HEIGHT
        },
        Player::Red
    ));
    assert!(grid.is_connected_to_base(
        Vec2 {
            x: -MAP_WIDTH + 1.0,
            y: -MAP_HEIGHT + 1.0
        },
        Player::Red
    ));
    assert!(!grid.is_connected_to_base(
        Vec2 {
            x: -MAP_WIDTH + 2.0,
            y: -MAP_HEIGHT
        },
        Player::Blue
    ));
}
