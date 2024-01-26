use strum::Display;

use crate::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_menu)
            .add_systems(Update, (open_menu, close_menu));
    }
}

#[derive(Component)]
pub struct Menu;

#[derive(Component, Display)]
pub enum MenuOption {
    Upgrade,
    ConvertFarm,
    Attack, //TODO
}

fn create_menu(mut commands: Commands) {
    let text_style = TextStyle {
        font_size: 30.0,
        color: Color::WHITE,
        ..default()
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(60.0),
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(250.),
                            border: UiRect::all(Val::Px(2.)),
                            ..default()
                        },
                        background_color: Color::rgb(0.65, 0.65, 0.65).into(),
                        visibility: Visibility::Hidden,
                        ..default()
                    },
                    Menu,
                ))
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.),
                                ..default()
                            },
                            background_color: Color::rgb(0.15, 0.15, 0.15).into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle {
                                    text: Text::from_sections([
                                        TextSection {
                                            value: "Tile: Empty\n".to_string(),
                                            style: text_style.clone(),
                                        },
                                        TextSection {
                                            value: "Level: 0\n".to_string(),
                                            style: text_style.clone(),
                                        },
                                        TextSection {
                                            value: "Health: 0\n".to_string(),
                                            style: text_style.clone(),
                                        },
                                        TextSection {
                                            value: "Option".to_string(),
                                            style: text_style.clone(),
                                        },
                                    ]),
                                    ..default()
                                },
                                Label,
                            ));
                        });
                });
        });
}

fn open_menu(
    mouse: Res<GridMouse>,
    mut events: EventReader<TileEvent>,
    tile_query: Query<(&Position, &Tile, &Level, &Owned, &Health)>,
    mut menu_root_query: Query<&mut Visibility, With<Menu>>,
    mut menu_query: Query<(Entity, &mut Text), With<Label>>,
    turn: Res<TurnCounter>,
    mut commands: Commands,
) {
    let Some(&TileEvent::SelectEvent(selected_position)) = events.read().next() else {
        return;
    };

    if let Some(Visibility::Visible) = menu_root_query.iter().next() {
        return;
    }

    info!("Opening menu");

    if let Some((_, Tile(tile_type), Level(level), &Owned(Some(owner)), Health(hp))) = tile_query
        .iter()
        .find(|(pos, ..)| pos.as_grid_index() == selected_position)
    {
        if owner != turn.player() {
            return;
        }
        info!("Menu for tile at {:?}", mouse.as_position());

        if let Some((ent, mut text)) = menu_query.iter_mut().next() {
            text.sections[0].value = format!("Tile: {:?}\n", tile_type);
            match tile_type {
                TileType::Occupied(PlayerTile::Tile, _) => {
                    text.sections[1].value = format!("Level: {}\n", level);
                    text.sections[2].value = format!("Health: {}\n", hp);
                }
                TileType::Occupied(PlayerTile::Base, _) => {
                    text.sections[2].value = format!("Health: {}\n", hp);
                }
                _ => {}
            }

            let mut menu_entity = commands.entity(ent);
            let buttons = match tile_type {
                TileType::Occupied(PlayerTile::Tile, Terrain::None) => vec![
                    MenuOption::Attack,
                    MenuOption::Upgrade,
                    MenuOption::ConvertFarm,
                ],
                TileType::Occupied(PlayerTile::Tile, Terrain::Mountain) => {
                    vec![MenuOption::Attack, MenuOption::ConvertFarm]
                }
                TileType::Occupied(PlayerTile::Base, _) => vec![MenuOption::Attack],
                _ => vec![],
            };

            for option in buttons.iter() {
                menu_entity.with_children(|parent| {
                    parent
                        .spawn(ButtonBundle {
                            style: Style {
                                width: Val::Percent(100.),
                                height: Val::Px(50.),
                                margin: UiRect::axes(Val::Px(2.0), Val::Px(2.0)),
                                ..default()
                            },
                            background_color: Color::rgb(0.15, 0.15, 0.15).into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn(TextBundle {
                                text: Text::from_sections(vec![TextSection {
                                    value: option.to_string(),
                                    style: TextStyle {
                                        font_size: 30.0,
                                        color: Color::WHITE,
                                        ..default()
                                    },
                                }]),
                                ..default()
                            });
                        });
                });
            } // TODO: Make this way better

            if let Some(mut visibility) = menu_root_query.iter_mut().next() {
                *visibility = Visibility::Visible;
            }
        }
    }
}

fn close_menu(
    mut menu_root_query: Query<&mut Visibility, With<Menu>>,
    mut menu_query: Query<&mut Text, With<Label>>,
    mut events: EventReader<TileEvent>,
) {
    let Some(TileEvent::DeselectEvent) = events.read().next() else {
        return;
    };

    info!("Closing menu");

    if let Some(mut text) = menu_query.iter_mut().next() {
        text.sections[0].value = "Tile: Empty\n".to_string();
        text.sections[1].value = "Level: 0\n".to_string();
        text.sections[2].value = "Health: 0\n".to_string();

        if let Some(mut visibility) = menu_root_query.iter_mut().next() {
            *visibility = Visibility::Hidden;
        }
    }
}
