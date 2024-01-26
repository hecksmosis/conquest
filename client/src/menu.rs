use std::vec;

use crate::*;
use bevy_simple_text_input::TextInput;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WinCounter::default())
            .add_systems(OnEnter(ClientState::Menu), setup_menu)
            .add_systems(OnExit(ClientState::Menu), cleanup)
            .add_systems(
                Update,
                (menu_manager, update_player_wins).run_if(in_state(ClientState::Menu)),
            );
    }
}

#[derive(Resource, Default)]
pub struct WinCounter {
    wins: [usize; 2],
}

impl WinCounter {
    pub fn get(&self, player: Player) -> usize {
        self.wins[player as usize]
    }

    pub fn increment(&mut self, player: Player, points: usize) {
        self.wins[player as usize] += points;
    }
}

#[derive(Component)]
pub struct PlayerWinsText;

fn setup_menu(mut commands: Commands) {
    let text = commands
        .spawn(TextBundle {
            text: Text::from_section(
                "Play",
                TextStyle {
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..default()
                },
            ),
            ..default()
        })
        .id();

    let text_input = commands
        .spawn((
            NodeBundle {
                background_color: Color::rgb(0.1, 0.1, 0.1).into(),
                ..default()
            },
            TextInput::default(),
        ))
        .id();

    let button = commands
        .spawn(ButtonBundle {
            style: Style {
                display: Display::Grid,
                grid_template_columns: vec![],
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .insert_children(0, &[text_input, text])
        .id();

    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "Player Wins: 0",
                TextStyle {
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..default()
                },
            ),
            ..Default::default()
        },
        PlayerWinsText,
    ));

    // Create a root UI entity for the menu
    commands
        .spawn(NodeBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                min_width: Val::Percent(100.0),
                min_height: Val::Percent(100.0),
                ..default()
            },
            ..default()
        })
        .insert_children(0, &[button]);
}

pub fn cleanup(
    mut commands: Commands,
    mut entities: Query<Entity, Or<(With<Node>, With<PlayerWinsText>)>>,
) {
    for ent in entities.iter_mut() {
        commands.entity(ent).despawn_recursive()
    }
}

fn menu_manager(
    interaction: Query<(&Interaction, &Button), Changed<Interaction>>,
    mut state: ResMut<NextState<ClientState>>,
) {
    interaction
        .iter()
        .filter(|(i, _)| matches!(i, Interaction::Pressed))
        .for_each(|_| {
            state.set(ClientState::Lobby);
        })
}

fn update_player_wins(
    player_wins: Res<WinCounter>,
    mut query: Query<&mut Text, With<PlayerWinsText>>,
) {
    for mut text in query.iter_mut() {
        text.sections[0].value = format!("Player Wins: {}", player_wins.get(Player::Red));
    }
}
