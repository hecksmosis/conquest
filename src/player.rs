use strum::{Display, EnumIter, EnumString};

#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash, EnumString, Display, EnumIter)]
pub enum Player {
    #[strum(serialize = "red")]
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
