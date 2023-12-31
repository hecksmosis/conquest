use crate::*;

pub struct TurnPlugin;

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_event::<TurnEvent>()
        .insert_resource(TurnCounter::new()).add_systems(Update, switch_turn);
    }
}

#[derive(Resource)]
pub struct TurnCounter {
    turn: Player,
}

impl TurnCounter {
    pub fn new() -> Self {
        Self { turn: Player::Red }
    }

    pub fn next(&mut self) {
        self.turn = self.turn.other();
    }

    pub fn player(&self) -> Player {
        self.turn.clone()
    }
}

fn switch_turn(mut turn_events: EventReader<TurnEvent>, mut turn: ResMut<TurnCounter>, mut attack_controller: ResMut<AttackController>) {
    if turn_events.read().count() == 0 {
        return;
    }

    attack_controller.deselect();
    turn.next();
}