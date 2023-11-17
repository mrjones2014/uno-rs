#![deny(clippy::all, clippy::pedantic, rust_2018_idioms, clippy::unwrap_used)]

pub mod cards;

use cards::{UnoCard, UnoValue, UnoWildCard};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::{iter, usize};
use thiserror::Error;

pub const FULL_DECK_SIZE: usize = 108;
pub const PLAYER_STARTING_HAND_SIZE: usize = 108;

#[derive(Debug, Error)]
pub enum UnoError {
    /// Not enough cards to deal to this many players
    #[error("Too many players: max 4, attempted {0}")]
    TooManyPlayers(usize),
    /// Not enough cards left to deal
    #[error("No cards left")]
    NoCardsLeft,
}

#[derive(Serialize, Deserialize)]
pub struct UnoDeck(Vec<UnoCard>);

impl UnoDeck {
    /// Get a brand new, shuffled full Uno deck.
    #[must_use]
    pub fn new() -> Self {
        let mut cards: Vec<UnoCard> = Vec::with_capacity(FULL_DECK_SIZE);

        // 2 sets of 0-9s and special cards for each color, but only 1 zero card per color
        // so: all the card permutations
        cards.append(&mut UnoCard::color_permutations());
        // then all the card permutations except the zero cards
        cards.append(
            &mut UnoCard::color_permutations()
                .into_iter()
                .filter(|card| {
                    !matches!(
                        card,
                        UnoCard::Card {
                            value: UnoValue::Zero,
                            ..
                        }
                    )
                })
                .collect(),
        );

        (0..8).for_each(|i| {
            // then 4 each of wild and wild draw 4 cards
            cards.push(UnoCard::Wild(UnoWildCard::Unplayed { draw_4: i < 4 }));
        });

        cards.shuffle(&mut rand::thread_rng());

        UnoDeck(cards)
    }

    /// Draw a card from the deck (the last card in the deck is the "top").
    /// Returns [`std::option::Option::None`] is there are no cards left.
    pub fn draw_card(&mut self) -> Option<UnoCard> {
        self.0.pop()
    }

    /// Deal out cards to specified number of players
    ///
    /// # Errors
    ///
    /// Returns an error if there are no cards left in the deck.
    pub fn deal(&mut self, players: usize) -> Result<Vec<UnoDeck>, UnoError> {
        let mut player_hands = iter::repeat(Vec::<UnoCard>::with_capacity(7))
            .take(players)
            .collect::<Vec<_>>();

        for i in 0..(PLAYER_STARTING_HAND_SIZE * players) {
            // deal cards round-robin style to each player, one at a time
            player_hands[i % players].push(self.draw_card().ok_or(UnoError::NoCardsLeft)?);
        }

        Ok(player_hands.into_iter().map(UnoDeck).collect())
    }
}

impl Default for UnoDeck {
    fn default() -> Self {
        Self::new()
    }
}
