use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};
use thiserror::Error;

#[derive(Debug, Eq, PartialEq, EnumIter, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UnoValue {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Skip,
    Reverse,
    Draw2,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UnoWildCard {
    Played { draw_4: bool, color: UnoColor },
    Unplayed { draw_4: bool },
}

#[derive(Debug, Eq, PartialEq, EnumIter, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UnoColor {
    Red,
    Blue,
    Green,
    Yellow,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UnoCard {
    Card { color: UnoColor, value: UnoValue },
    Wild(UnoWildCard),
}

/// Errors describing why one card cannot be played on another.
#[derive(Debug, Error)]
pub enum UnoCardMatchError {
    /// It's a normal card and the color and value both don't match
    #[error("Card color and value both don't match.")]
    NoMatch,
    /// Wild card is in "unplayed" state (color not set)
    #[error("Wild card is in unplayed state (color not set)")]
    WildUnplayed,
}

impl UnoCard {
    /// Returns `Ok(())` if the card is playable, otherwise
    /// returns [`UnoPlayError`] describing why it cannot be played on
    /// `other_card`.
    ///
    /// # Errors
    ///
    /// Returns an error describing why the card cannot be played. [`UnoCardMatchError`]
    pub fn playable_on(&self, other_card: &UnoCard) -> Result<(), UnoCardMatchError> {
        match self {
            UnoCard::Card {
                color: self_color,
                value: self_value,
            } => match other_card {
                UnoCard::Card {
                    color: other_color,
                    value: other_value,
                } => {
                    if self_color == other_color || self_value == other_value {
                        Ok(())
                    } else {
                        Err(UnoCardMatchError::NoMatch)
                    }
                }
                // if the other card is wild, must match their color
                UnoCard::Wild(other_wild) => match other_wild {
                    UnoWildCard::Played {
                        color: other_color, ..
                    } => {
                        if self_color == other_color {
                            Ok(())
                        } else {
                            Err(UnoCardMatchError::NoMatch)
                        }
                    }
                    UnoWildCard::Unplayed { .. } => Err(UnoCardMatchError::WildUnplayed),
                },
            },
            // can always play a wild card
            UnoCard::Wild(self_wild) => match self_wild {
                UnoWildCard::Played { .. } => Ok(()),
                UnoWildCard::Unplayed { .. } => Err(UnoCardMatchError::WildUnplayed),
            },
        }
    }

    /// Get all permutations of `UnoCard::Card` by combining [`UnoColor`]s with [`UnoValue`]s.
    /// Returns a list of [`UnoCard`]s representing one each of:
    /// - All 4 colors, 0-9
    /// - All 4 colors:
    ///     - Skip
    ///     - Reverse
    ///     - Draw 2
    #[must_use]
    pub(crate) fn color_permutations() -> Vec<UnoCard> {
        UnoColor::iter()
            .flat_map(|color| UnoValue::iter().map(move |value| UnoCard::Card { color, value }))
            .collect::<Vec<_>>()
    }
}
