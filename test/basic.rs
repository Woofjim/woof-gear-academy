use gtest::{Program, System};
use io::*;

#[test]
fn pebbles_game_test() {
    let sys = System::new();

    let program = Program::from_file(&sys, "./target/wasm32-unknown-unknown/release/pebbles-game.wasm");

    let init_data = PebblesInit {
        difficulty: DifficultyLevel::Easy,  
        pebbles_count: 15,                
        max_pebbles_per_turn: 2,           
    };
    
    let _ = program.send_bytes(100001, &init_data); 

    let state: GameState = program.query(&(), 0);
    assert_eq!(state.pebbles_count, 15);
    assert_eq!(state.max_pebbles_per_turn, 2);
    assert_eq!(state.pebbles_remaining, 15);

    let user_action = PebblesAction::Turn(2);
    let _ = program.send_bytes(100002, &user_action);

    let event: PebblesEvent = program.query(&(), 0);
    match event {
        PebblesEvent::CounterTurn(pebbles) => {
            assert!(pebbles >= 1 && pebbles <= 2); 
        }
        _ => panic!("Expected CounterTurn event, but got something else."),
    }

    let state_after_user_turn: GameState = program.query(&(), 0);
    assert_eq!(state_after_user_turn.pebbles_remaining, 13); 

    let give_up_action = PebblesAction::GiveUp;
    let _ = program.send_bytes(100002, &give_up_action);

    let final_event: PebblesEvent = program.query(&(), 0);
    match final_event {
        PebblesEvent::Won(Player::Program) => {}
        _ => panic!("Expected Program to win, but got something else."),
    }

    let final_state: GameState = program.query(&(), 0);
    assert_eq!(final_state.winner, Some(Player::Program));
}
