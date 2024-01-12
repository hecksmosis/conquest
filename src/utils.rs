use std::usize;

use crate::*;
use bevy::window::PrimaryWindow;

pub fn get_rectified_mouse_position(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) -> Option<Vec2> {
    let (camera, camera_transform) = camera_query.single();

    let viewport_position = q_windows.single().cursor_position()?;
    let position = camera.viewport_to_world_2d(camera_transform, viewport_position)?;

    Some((position / TILE_SIZE).floor() * TILE_SIZE)
}

pub fn get_vec_from_index(index: usize) -> Vec2 {
    Vec2::new(
        (index % (MAP_WIDTH as usize * 2)) as f32,
        (index / (MAP_WIDTH as usize * 2)) as f32,
    )
}

fn attack_is_valid(origin: Vec2, target: Vec2, level: usize) -> bool {
    let diff = target - origin;

    info!("{}, level: {}", diff, level);

    match level {
        1 | 3 => [
            Vec2::new(1.0, 0.0),
            Vec2::new(-1.0, 0.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(0.0, -1.0),
        ]
        .contains(&diff),
        2 => [
            Vec2::new(1.0, 0.0),
            Vec2::new(-1.0, 0.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(0.0, -1.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(-2.0, 0.0),
            Vec2::new(0.0, 2.0),
            Vec2::new(0.0, -2.0),
        ]
        .contains(&diff),
        _ => false,
    }
}

pub fn get_attack_targets(origin: Option<Vec2>, target: Vec2, level: usize) -> Vec<Vec2> {
    let Some(origin) = origin else {
        return vec![];
    };
    if !attack_is_valid(origin, target, level) {
        return vec![];
    }

    let diff = target - origin;
    let direction = diff / diff.length();
    match level {
        2 => vec![origin + direction, origin + direction * 2.0],
        3 => vec![
            origin + direction,
            origin + direction + Vec2::new(direction.y, -direction.x),
            origin + direction + Vec2::new(-direction.y, direction.x),
        ],
        _ => vec![target],
    }
}

pub fn count_farms(farms: &Query<(&Tile, &Owned, &Level)>) -> [usize; 2] {
    farms
        .iter()
        .filter(|(Tile(tile_type), _, _)| tile_type.is_farm())
        .fold([100; 2], |mut total, (_, Owned(owner), Level(level))| {
            //TODO: change 100 to 1
            let Some(player) = owner else {
                return total;
            };

            total[*player as usize] += level;
            total
        })
}

pub fn get_selected_grid_position(
    attack_controller: &Res<AttackController>,
    grid: &Res<TileGrid>,
    mouse: &Res<GridMouse>,
    player: Player,
) -> Option<Vec2> {
    attack_controller
        .selected
        .or_else(|| grid.get_any_connected(mouse.grid_position(), player))
}
