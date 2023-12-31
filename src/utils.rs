use crate::*;
use bevy::window::PrimaryWindow;

pub fn get_rectified_mouse_position(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) -> Option<Vec2> {
    let (camera, camera_transform) = camera_query.single();

    let Some(mouse_position) = q_windows.single().cursor_position() else {
        return None;
    };

    let Some(position) = camera.viewport_to_world_2d(camera_transform, mouse_position) else {
        return None;
    };

    Some((position / TILE_SIZE).floor() * TILE_SIZE)
}

pub fn get_vec_from_index(index: usize) -> Vec2 {
    Vec2::new(
        (index % (MAP_WIDTH as usize * 2)) as f32,
        (index / (MAP_WIDTH as usize * 2)) as f32,
    )
}

pub fn attack_is_valid(origin: Vec2, target: Vec2, level: usize) -> bool {
    let diff = target - origin;

    info!("{}", diff);

    match level {
        1 => [
            Vec2::new(1.0, 0.0),
            Vec2::new(-1.0, 0.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(0.0, -1.0),
        ]
        .contains(&diff),
        _ => false,
    }
}
