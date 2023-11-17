#![deny(clippy::all, clippy::pedantic, rust_2018_idioms, clippy::unwrap_used)]

pub mod cards;

use cards::{UnoCard, UnoCardMatchError, UnoValue, UnoWildCard};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::{borrow::BorrowMut, iter, usize};
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
    /// Invalid player number, must be 1 through players.len()
    #[error("Invalid player number")]
    InvalidPlayerNumber,
    /// The player doesn't have the specified card
    #[error("The player does not have the specified card in their hand")]
    Cheating,
    /// Chosen card doesn't match discard pile top card
    #[error("Chosen card doesn't match the top card of the discard pile")]
    CardNotPlayable(#[from] UnoCardMatchError),
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

    /// Put the discard deck back into the deck and reshuffle.
    #[must_use]
    pub fn from_discard(discard_deck: &UnoDeck) -> UnoDeck {
        let mut cards = discard_deck.0.clone();
        cards.shuffle(&mut rand::thread_rng());
        UnoDeck(cards)
    }
}

impl Default for UnoDeck {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TurnDirection {
    Clockwise,
    CounterClockwise,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnoGameState {
    pub main_deck: UnoDeck,
    pub discard_deck: UnoDeck,
    pub player_hands: Vec<UnoDeck>,
    pub turn_direction: TurnDirection,
    pub current_turn: usize,
}

impl UnoGameState {
    /// Initialize a new game with the specified number of players.
    ///
    /// # Errors
    ///
    /// Errors if there are not enough cards. Should never happen
    /// since we're making a brand new deck.
    pub fn new(players: usize) -> Result<Self, UnoError> {
        let mut main_deck = UnoDeck::new();
        // deal to players first
        let player_hands = main_deck.deal(players)?;
        // draw the first card to start the game
        let discard_deck = UnoDeck(vec![main_deck.draw_card().ok_or(UnoError::NoCardsLeft)?]);
        Ok(Self {
            main_deck,
            discard_deck,
            player_hands,
            turn_direction: TurnDirection::Clockwise,
            current_turn: 0, // TODO dice roll for who goes first
        })
    }

    /// Draw `n` cards, handling recycling the main deck from the discard
    /// deck if it runs out.
    ///
    /// # Panics
    ///
    /// Panics if there are not enough cards in the discard and main decks combined. TODO fix
    /// this.
    pub fn draw_n_cards(&mut self, n: usize) -> Vec<UnoCard> {
        let mut cards = vec![];
        for _ in 0..n {
            if let Some(card) = self.main_deck.draw_card() {
                cards.push(card);
            } else {
                // move all but top card to main deck and shuffle
                let top_card = self
                    .discard_deck
                    .0
                    .pop()
                    .expect("No cards left -- everyone has huge hands?"); // TODO handle
                                                                          // panic
                self.main_deck.0.append(&mut self.discard_deck.0);
                self.discard_deck = UnoDeck(vec![top_card]);
                cards.push(
                    self.main_deck
                        .draw_card()
                        .expect("There should be cards now."),
                );
            }
        }

        cards
    }

    /// Try to set the next game state by playing the specified card. Does not modify
    /// state if turn validation fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - `whos_turn` is not a valid player number
    /// - `which_card` cannot be played on the top of the current discard deck
    /// - The specified player does not have the specified card in their hand
    pub fn try_next(
        &mut self,
        whos_turn: usize,
        which_card: &UnoCard,
    ) -> Result<&mut Self, UnoError> {
        // TODO track and account for turn direction and skip turns
        if whos_turn > self.player_hands.len() - 1 {
            return Err(UnoError::InvalidPlayerNumber);
        }

        let player_hand = &self.player_hands[whos_turn];
        let Some(card_idx) = player_hand.0.iter().position(|card| card == which_card) else {
            return Err(UnoError::Cheating);
        };

        let top_card = &self.discard_deck.0[self.discard_deck.0.len() - 1];
        which_card.playable_on(top_card)?;

        let card = self.player_hands[whos_turn].0.remove(card_idx);
        self.discard_deck.0.push(card);

        match card {
            UnoCard::Card { value, .. } => match value {
                UnoValue::Skip => {
                    // skip player
                    self.current_turn = (self.current_turn + 2) % self.player_hands.len();
                }
                UnoValue::Reverse => {
                    // reverse
                    self.turn_direction = match self.turn_direction {
                        TurnDirection::Clockwise => {
                            self.current_turn = if self.current_turn == 0 {
                                self.player_hands.len() - 1
                            } else {
                                self.current_turn - 1
                            };
                            TurnDirection::CounterClockwise
                        }
                        TurnDirection::CounterClockwise => {
                            self.current_turn = (self.current_turn + 1) % self.player_hands.len();
                            TurnDirection::Clockwise
                        }
                    }
                }
                UnoValue::Draw2 => {
                    // advance turn
                    self.current_turn = (self.current_turn + 1) % self.player_hands.len();
                    // make them draw cards
                    let mut drawn = self.draw_n_cards(2);
                    self.player_hands[self.current_turn].0.append(&mut drawn);
                    // draw 2 is also skip
                    self.current_turn = (self.current_turn + 1) % self.player_hands.len();
                }
                _ => {}
            },
            UnoCard::Wild(wild) => match wild {
                UnoWildCard::Played { draw_4, .. } => {
                    if draw_4 {
                        // advance turn
                        self.current_turn = (self.current_turn + 1) % self.player_hands.len();
                        // make them draw cards
                        // TODO handle discard deck recycle
                        let mut drawn = self.draw_n_cards(4);
                        self.player_hands[self.current_turn].0.append(&mut drawn);
                        // draw 4 is also skip
                        self.current_turn = (self.current_turn + 1) % self.player_hands.len();
                    }
                }
                UnoWildCard::Unplayed { .. } => {
                    unreachable!("Already validated by playable_on function above")
                }
            },
        };

        Ok(self)
    }
}
