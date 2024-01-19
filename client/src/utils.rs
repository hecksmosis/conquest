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