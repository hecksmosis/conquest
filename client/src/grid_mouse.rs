use crate::*;

pub struct GridMousePlugin;

impl Plugin for GridMousePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_mouse_position)
            .insert_resource(GridMouse::default());
    }
}

#[derive(Resource, Default)]
pub struct GridMouse {
    position: Vec2,
}

impl GridMouse {
    /// Returns the position of the mouse as a grid index.
    pub fn grid_position(&self) -> Vec2 {
        self.position / TILE_SIZE
    }
}

fn update_mouse_position(
    mut mouse: ResMut<GridMouse>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    let Some(rectified_position) = get_rectified_mouse_position(camera_query, q_windows) else {
        return;
    };

    mouse.position = rectified_position;
}
