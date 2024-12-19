#![no_std]

use gstd::{exec, msg, prelude::*};
use io::*;

static mut GAME_STATE: Option<GameState> = None;

#[no_mangle]
extern "C" fn init() {
    let init: PebblesInit = msg::load().expect("Unable to load PebblesInit");

    assert!(
        init.pebbles_count > 0,
        "Pebbles count must be greater than 0"
    );
    assert!(
        init.max_pebbles_per_turn > 0,
        "Max pebbles per turn must be greater than 0"
    );

    let first_player = if get_random_u32() % 2 == 0 {
        Player::User
    } else {
        Player::Program
    };

    let game_state = GameState {
        pebbles_count: init.pebbles_count,
        max_pebbles_per_turn: init.max_pebbles_per_turn,
        pebbles_remaining: init.pebbles_count,
        difficulty: init.difficulty,
        first_player: first_player.clone(),
        winner: None,
    };

    unsafe {
        GAME_STATE = Some(game_state);
    }

    if matches!(first_player, Player::Program) {
        program_turn();
    }
}

#[no_mangle]
extern "C" fn handle() {
    let action: PebblesAction = msg::load().expect("Unable to load PebblesAction");

    unsafe {
        let game_state = GAME_STATE.as_mut().expect("Game state is not initialized");

        match action {
            PebblesAction::Turn(pebbles) => {
                assert!(
                    pebbles > 0 && pebbles <= game_state.max_pebbles_per_turn,
                    "Invalid number of pebbles"
                );
                assert!(
                    pebbles <= game_state.pebbles_remaining,
                    "Not enough pebbles remaining"
                );

                game_state.pebbles_remaining -= pebbles;
                if game_state.pebbles_remaining == 0 {
                    game_state.winner = Some(Player::User);
                    msg::reply(PebblesEvent::Won(Player::User), 0).expect("Unable to reply");
                    return;
                }

                program_turn();
            }
            PebblesAction::GiveUp => {
                game_state.winner = Some(Player::Program);
                msg::reply(PebblesEvent::Won(Player::Program), 0).expect("Unable to reply");
            }
            PebblesAction::Restart {
                difficulty,
                pebbles_count,
                max_pebbles_per_turn,
            } => {
                assert!(pebbles_count > 0, "Pebbles count must be greater than 0");
                assert!(
                    max_pebbles_per_turn > 0,
                    "Max pebbles per turn must be greater than 0"
                );

                *game_state = GameState {
                    pebbles_count,
                    max_pebbles_per_turn,
                    pebbles_remaining: pebbles_count,
                    difficulty,
                    first_player: if get_random_u32() % 2 == 0 {
                        Player::User
                    } else {
                        Player::Program
                    },
                    winner: None,
                };

                if matches!(game_state.first_player, Player::Program) {
                    program_turn();
                }
            }
        }
    }
}

#[no_mangle]
extern "C" fn state() {
    unsafe {
        let game_state = GAME_STATE.as_ref().expect("Game state is not initialized");
        msg::reply(game_state, 0).expect("Unable to reply with state");
    }
}

fn program_turn() {
    unsafe {
        let game_state = GAME_STATE.as_mut().expect("Game state is not initialized");

        let pebbles_to_remove = match game_state.difficulty {
            DifficultyLevel::Easy => 1 + (get_random_u32() % game_state.max_pebbles_per_turn),
            DifficultyLevel::Hard => find_best_move(
                game_state.pebbles_remaining,
                game_state.max_pebbles_per_turn,
            ),
        };

        game_state.pebbles_remaining -= pebbles_to_remove;

        if game_state.pebbles_remaining == 0 {
            game_state.winner = Some(Player::Program);
            msg::reply(PebblesEvent::Won(Player::Program), 0).expect("Unable to reply");
        } else {
            msg::reply(PebblesEvent::CounterTurn(pebbles_to_remove), 0).expect("Unable to reply");
        }
    }
}

fn find_best_move(pebbles_remaining: u32, max_pebbles_per_turn: u32) -> u32 {
    for i in 1..=max_pebbles_per_turn {
        if (pebbles_remaining as i32 - i as i32 - 1) % (max_pebbles_per_turn as i32 + 1) == 0 {
            return i;
        }
    }
    1
}

#[cfg(not(test))]
fn get_random_u32() -> u32 {
    let salt = msg::id();
    let (hash, _num) = exec::random(salt.into()).expect("get_random_u32(): random call failed");
    u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
}

#[cfg(test)]
fn get_random_u32() -> u32 {
    42
}
