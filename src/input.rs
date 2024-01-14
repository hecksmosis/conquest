use crate::*;

const INPUTS: [InputType; 6] = [
    InputType::Mouse(MouseButton::Right),
    InputType::Mouse(MouseButton::Left),
    InputType::Key(KeyCode::Space),
    InputType::Key(KeyCode::Return),
    InputType::Key(KeyCode::M),
    InputType::Key(KeyCode::W),
];

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct GameInput(u8);

impl GameInput {
    pub fn get_buttons(&self) -> impl Iterator<Item = MouseButton> + '_ {
        INPUTS
            .iter()
            .enumerate()
            .filter(|(flag, _)| self.0 & (1 << flag) != 0)
            .filter_map(|(_, input_type)| match input_type {
                InputType::Mouse(button) => Some(*button),
                _ => None,
            })
    }

    pub fn get_keys(&self) -> impl Iterator<Item = KeyCode> + '_ {
        INPUTS
            .iter()
            .enumerate()
            .filter(|(flag, _)| self.0 & (1 << flag) != 0)
            .filter_map(|(_, input_type)| match input_type {
                InputType::Key(key) => Some(*key),
                _ => None,
            })
    }
}

enum InputType {
    Mouse(MouseButton),
    Key(KeyCode),
}

pub fn read_local_inputs(
    mut commands: Commands,
    (buttons, keys): (Res<Input<MouseButton>>, Res<Input<KeyCode>>),
    local_players: Res<LocalPlayers>,
) {
    let local_inputs = local_players
        .0
        .iter()
        .map(|handle| {
            let input = INPUTS
                .iter()
                .enumerate()
                .filter(|(_, input_type)| match input_type {
                    InputType::Mouse(button) => buttons.just_pressed(*button),
                    InputType::Key(key) => keys.just_pressed(*key),
                })
                .fold(0, |acc, (flag, _)| acc | 1 << flag);

            (*handle, GameInput(input))
        })
        .collect();

    commands.insert_resource(LocalInputs::<Config>(local_inputs));
}
