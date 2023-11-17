#![deny(clippy::all, clippy::pedantic, rust_2018_idioms, clippy::unwrap_used)]

use rand::seq::SliceRandom;
use std::{iter, usize};
use strum::{EnumIter, IntoEnumIterator};

pub const FULL_DECK_SIZE: usize = 108;
pub const USER_HAND_SIZE: usize = 108;

/// Numeric card
#[derive(EnumIter, Clone, Copy)]
pub enum UnoNumeric {
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
}

#[derive(EnumIter, Clone, Copy)]
pub enum UnoSpecial {
    Skip,
    Reverse,
    Draw2,
}

#[derive(Clone, Copy)]
pub enum UnoValue {
    Numeric(UnoNumeric),
    Special(UnoSpecial),
}

#[derive(Clone, Copy)]
pub enum UnoWild {
    Wild,
    WildDraw4,
}

#[derive(Clone, Copy)]
pub enum UnoCard {
    Red(UnoValue),
    Blue(UnoValue),
    Green(UnoValue),
    Yellow(UnoValue),
    Wild(UnoWild),
}

impl UnoCard {
    /// Given the [`UnoValue`], generate all
    /// Uno color cards for the value.
    fn color_permutations(value: UnoValue) -> [UnoCard; 4] {
        [
            UnoCard::Red(value),
            UnoCard::Yellow(value),
            UnoCard::Blue(value),
            UnoCard::Green(value),
        ]
    }
}

/// Collection of cards. Can be main deck or user hand.
pub struct UnoDeck(Vec<UnoCard>);

pub enum UnoError {
    /// Not enough cards to deal to this many players
    TooManyPlayers(usize),
    /// Not enough cards left to deal
    NoCardsLeft,
}

impl UnoDeck {
    /// Create a new full deck (all 108 cards). You can then deal the cards from this deck.
    /// Returns the deck pre-shuffled.
    #[must_use]
    pub fn new() -> Self {
        let mut cards = Vec::with_capacity(FULL_DECK_SIZE);

        for i in 0..2 {
            // two sets of numeric cards per color, but only one 0 card per color
            for number in UnoNumeric::iter().skip(i) {
                cards.append(&mut UnoCard::color_permutations(UnoValue::Numeric(number)).to_vec());
            }

            // two of each special card per color
            for special in UnoSpecial::iter() {
                cards.append(&mut UnoCard::color_permutations(UnoValue::Special(special)).to_vec());
            }
        }

        // 4 of each wild card type (plain wild or draw 4)
        cards.append(&mut iter::repeat(UnoCard::Wild(UnoWild::Wild)).take(4).collect());
        cards.append(
            &mut iter::repeat(UnoCard::Wild(UnoWild::WildDraw4))
                .take(4)
                .collect(),
        );

        cards.shuffle(&mut rand::thread_rng());

        UnoDeck(cards)
    }

    pub fn pop(&mut self) -> Option<UnoCard> {
        self.0.pop()
    }

    /// Deal a deck to the given number of players. Deals cards from the deck
    /// and returns a `Vec<UnoDeck>` where the vector length is equal to the
    /// number of given players (one deck per player).
    /// Deals in place (main deck is mutated).
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn deal(&mut self, players: usize) -> Result<Vec<UnoDeck>, UnoError> {
        if players > 4 {
            return Err(UnoError::TooManyPlayers(players));
        }

        let mut player_decks: Vec<Vec<UnoCard>> = Vec::with_capacity(players);
        // initialize vecs
        (0..players).for_each(|i| {
            player_decks[i] = Vec::with_capacity(USER_HAND_SIZE);
        });

        for _ in 0..USER_HAND_SIZE {
            for j in 0..players {
                let Some(player_deck) = player_decks.get_mut(j) else {
                    unreachable!()
                };

                player_deck.push(self.pop().ok_or(UnoError::NoCardsLeft)?);
            }
        }

        Ok(player_decks.into_iter().map(UnoDeck).collect())
    }

    /// Check if the given card can be played on top of the card on top of the current deck.
    /// "Top" of the deck is the end of the `Vec<UnoCard>`.
    pub fn can_play_card(&self, _next_card: UnoCard) {
        unimplemented!("TODO")
    }
}

impl Default for UnoDeck {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_108_card_uno_deck() {
        let deck = UnoDeck::new();
        assert_eq!(FULL_DECK_SIZE, deck.0.len());

        // 19 of each color numeric cards (set of 0-9 and set of 1-9)
        assert_eq!(
            19,
            deck.0
                .iter()
                .filter(|card| matches!(card, UnoCard::Red(UnoValue::Numeric(_))))
                .collect::<Vec<_>>()
                .len()
        );
        assert_eq!(
            19,
            deck.0
                .iter()
                .filter(|card| matches!(card, UnoCard::Yellow(UnoValue::Numeric(_))))
                .collect::<Vec<_>>()
                .len()
        );
        assert_eq!(
            19,
            deck.0
                .iter()
                .filter(|card| matches!(card, UnoCard::Blue(UnoValue::Numeric(_))))
                .collect::<Vec<_>>()
                .len()
        );
        assert_eq!(
            19,
            deck.0
                .iter()
                .filter(|card| matches!(card, UnoCard::Green(UnoValue::Numeric(_))))
                .collect::<Vec<_>>()
                .len()
        );

        // 24 special cards (skip/reverse/draw 2), 6 of each color
        assert_eq!(
            6,
            deck.0
                .iter()
                .filter(|card| matches!(card, UnoCard::Red(UnoValue::Special(_))))
                .collect::<Vec<_>>()
                .len()
        );
        assert_eq!(
            6,
            deck.0
                .iter()
                .filter(|card| matches!(card, UnoCard::Yellow(UnoValue::Special(_))))
                .collect::<Vec<_>>()
                .len()
        );
        assert_eq!(
            6,
            deck.0
                .iter()
                .filter(|card| matches!(card, UnoCard::Blue(UnoValue::Special(_))))
                .collect::<Vec<_>>()
                .len()
        );
        assert_eq!(
            6,
            deck.0
                .iter()
                .filter(|card| matches!(card, UnoCard::Green(UnoValue::Special(_))))
                .collect::<Vec<_>>()
                .len()
        );

        // 4 Wilds and 4 Wild Draw 4s
        assert_eq!(
            4,
            deck.0
                .iter()
                .filter(|card| matches!(card, UnoCard::Wild(UnoWild::Wild)))
                .collect::<Vec<_>>()
                .len()
        );
        assert_eq!(
            4,
            deck.0
                .iter()
                .filter(|card| matches!(card, UnoCard::Wild(UnoWild::WildDraw4)))
                .collect::<Vec<_>>()
                .len()
        );
    }
}
