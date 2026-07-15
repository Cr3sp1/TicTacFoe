use std::time::{Duration, Instant};

use tic_tac_foe::{
    app::App,
    game::{Board, GameVariant, Mark},
    network::NetworkStatus,
    scenes::{GameMode, Scene},
    utils::Position,
};

const TIMEOUT: Duration = Duration::from_secs(10);

#[test]
fn online_ultimate_apps_exchange_moves_and_stay_synchronized() {
    let mut host = App::new();
    let mut joiner = App::new();

    host.host_online_match(GameVariant::Ultimate).unwrap();
    let ticket = wait_for_ticket(&mut host);
    joiner
        .join_online_match(ticket, GameVariant::Ultimate)
        .unwrap();
    wait_for_connection(&mut host, &mut joiner);

    assert_online_ultimate_player(&host, Mark::X);
    assert_online_ultimate_player(&joiner, Mark::O);

    play_selected_move(&mut host, 0, 0, 1, 2);
    wait_for_turn(&mut host, &mut joiner, 1);

    play_selected_move(&mut joiner, 1, 2, 0, 0);
    wait_for_turn(&mut host, &mut joiner, 2);

    play_selected_move(&mut host, 0, 0, 2, 1);
    wait_for_turn(&mut host, &mut joiner, 3);

    let host_game = online_ultimate_game(&host);
    let joiner_game = online_ultimate_game(&joiner);
    assert_eq!(host_game.big_board, joiner_game.big_board);
    assert_eq!(host_game.active_player, joiner_game.active_player);
    assert_eq!(host_game.turn, joiner_game.turn);
    assert_eq!(host_game.big_board.active_board, Some((2, 1)));
    assert_eq!(host_game.big_board.get_board(0, 0).get(1, 2), Some(Mark::X));
    assert_eq!(host_game.big_board.get_board(1, 2).get(0, 0), Some(Mark::O));
    assert_eq!(host_game.big_board.get_board(0, 0).get(2, 1), Some(Mark::X));
}

fn wait_for_ticket(host: &mut App) -> String {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        host.poll_network_events();
        if let NetworkStatus::Hosting { ticket, .. } = &host.network_status {
            return ticket.clone();
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for host ticket"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn wait_for_connection(host: &mut App, joiner: &mut App) {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        host.poll_network_events();
        joiner.poll_network_events();
        if matches!(host.network_status, NetworkStatus::Connected { .. })
            && matches!(joiner.network_status, NetworkStatus::Connected { .. })
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out connecting apps: host={:?}, joiner={:?}",
            host.network_status,
            joiner.network_status
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn play_selected_move(
    app: &mut App,
    board_row: usize,
    board_col: usize,
    cell_row: usize,
    cell_col: usize,
) {
    let Scene::PlayingUTT(game) = &mut app.current_scene else {
        panic!("expected online ultimate game");
    };
    game.selected_board = Position {
        row: board_row,
        col: board_col,
    };
    game.selected_cell = Some(Position {
        row: cell_row,
        col: cell_col,
    });
    app.handle_enter();
}

fn wait_for_turn(host: &mut App, joiner: &mut App, expected_turn: u32) {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        host.poll_network_events();
        joiner.poll_network_events();
        if online_ultimate_game(host).turn == expected_turn
            && online_ultimate_game(joiner).turn == expected_turn
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for turn {expected_turn}: host={}, joiner={}",
            online_ultimate_game(host).turn,
            online_ultimate_game(joiner).turn
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn assert_online_ultimate_player(app: &App, expected_mark: Mark) {
    let game = online_ultimate_game(app);
    assert_eq!(game.mode, GameMode::OnlinePvP(expected_mark));
}

fn online_ultimate_game(app: &App) -> &tic_tac_foe::scenes::GamePlayUTT {
    let Scene::PlayingUTT(game) = &app.current_scene else {
        panic!("expected online ultimate game");
    };
    game
}
