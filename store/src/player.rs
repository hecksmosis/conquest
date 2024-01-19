use crate::*;
use strum::{Display, EnumIter, EnumString};

#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash, EnumString, Display, EnumIter, Deserialize, Serialize, Default)]
pub enum Player {
    #[strum(serialize = "red")]
    #[default]
    Red,
    #[strum(serialize = "blue")]
    Blue,
}

impl Player {
    pub fn other(&self) -> Player {
        match self {
            Player::Red => Player::Blue,
            Player::Blue => Player::Red,
        }
    }
}

impl From<usize> for Player {
    fn from(i: usize) -> Self {
        match i {
            0 => Player::Red,
            1 => Player::Blue,
            _ => panic!("Invalid player index: {}", i),
        }
    }
}
