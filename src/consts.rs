use bevy::{render::color::Color, ui::Val};

pub const MAP_WIDTH: f32 = 8.0;
pub const MAP_HEIGHT: f32 = 4.0;
pub const TILE_SIZE: f32 = 50.0;
pub const SCOREBOARD_FONT_SIZE: f32 = 40.0;
pub const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);
pub const SCOREBOARD_TEXT_PADDING_2: Val = Val::Px(55.0);
pub const TEXT_COLOR: Color = Color::rgb(0.0, 0.0, 0.0);
pub const SCORE_COLOR: Color = Color::rgb(0.0, 0.0, 0.0);
pub const MAX_MOUNTAIN_COUNT: usize = 5;
pub const MAX_WATER_COUNT: usize = 4;
pub const GRID_SIZE: usize = (MAP_HEIGHT * MAP_WIDTH * 4.0 + 1.0) as usize;
