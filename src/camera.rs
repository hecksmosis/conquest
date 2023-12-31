use crate::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(Update, zoom);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn zoom(mut query_camera: Query<&mut OrthographicProjection>, keys: Res<Input<KeyCode>>) {
    if !keys.any_just_pressed([KeyCode::Minus, KeyCode::A]) {
        return;
    }

    let mut projection = query_camera.single_mut();

    if keys.just_pressed(KeyCode::Minus) {
        projection.scale *= 1.25;
    } else if keys.just_pressed(KeyCode::A) {
        projection.scale /= 1.25;
    }
}
