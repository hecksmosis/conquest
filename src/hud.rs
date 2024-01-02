use crate::*;

pub struct HUDPlugin;

impl Plugin for HUDPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_hud)
            .add_systems(Update, (count_farms, set_turn));
    }
}

#[derive(Component, Clone, Debug)]
pub struct FarmText;

#[derive(Component, Clone, Debug)]
pub struct TurnText;

#[derive(Component, Clone, Debug)]
pub struct PlacementModeText;

fn setup_hud(mut commands: Commands) {
    [
        ("red", SCOREBOARD_TEXT_PADDING),
        ("blue", SCOREBOARD_TEXT_PADDING_2),
    ]
    .iter()
    .for_each(|&(m, margin)| {
        let text_style = TextStyle {
            font_size: SCOREBOARD_FONT_SIZE,
            color: SCORE_COLOR,
            ..default()
        };
        commands.spawn((
            TextBundle::from_sections([
                TextSection {
                    value: format!("Farms {m}: "),
                    style: text_style.clone(),
                },
                TextSection {
                    value: "0".to_string(),
                    style: text_style.clone(),
                },
            ])
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: margin,
                left: SCOREBOARD_TEXT_PADDING,
                ..default()
            }),
            FarmText,
        ));
    });

    // Create turn text
    commands.spawn((
        TextBundle::from_sections([
            TextSection {
                value: "Turn: ".to_string(),
                style: TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            },
            TextSection {
                value: "Red".to_string(),
                style: TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: Color::rgb(1.0, 0.0, 0.0),
                    ..default()
                },
            },
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: SCOREBOARD_TEXT_PADDING,
            right: SCOREBOARD_TEXT_PADDING,
            ..default()
        }),
        TurnText,
    ));

    // Placement mode text
    commands.spawn((
        TextBundle::from_sections([
            TextSection {
                value: "Placement Mode: ".to_string(),
                style: TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            },
            TextSection {
                value: "None".to_string(),
                style: TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            },
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: SCOREBOARD_TEXT_PADDING_2,
            right: SCOREBOARD_TEXT_PADDING,
            ..default()
        }),
        PlacementModeText,
    ));
}

fn count_farms(farms: Res<FarmCounter>, mut query: Query<(&mut Text, &FarmText)>) {
    for (i, (mut text, _)) in query.iter_mut().enumerate() {
        text.sections[1].value = farms.available_farms(i.into()).to_string();
    }
}

fn set_turn(mut query: Query<&mut Text, With<TurnText>>, turn: Res<TurnCounter>) {
    for mut text in query.iter_mut() {
        (text.sections[1].value, text.sections[1].style.color) = match turn.player() {
            Player::Red => ("Red".to_string(), Color::rgb(1.0, 0.0, 0.0)),
            Player::Blue => ("Blue".to_string(), Color::rgb(0.0, 0.0, 1.0)),
        };
    }
}
