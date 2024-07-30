use std::io::prelude::*;
use std::time::Duration;

use cozy_chess::{Board, Color};
use cozy_chess::util::{parse_uci_move, display_uci_move};

mod engine;
mod bench;

use engine::{Engine, SearchLimits};

fn main() {
    if std::env::args().nth(1).as_deref() == Some("bench") {
        bench::run_bench();
        return;
    }

    let mut init_pos = Board::startpos();
    let mut current_pos = Board::startpos();
    let mut moves_played = Vec::new();
    let mut engine = Engine::new();

    for line in std::io::stdin().lines() {
        let line = line.expect("failed to read line");
        let tokens = line.split_ascii_whitespace().collect::<Vec<_>>();

        let Some(&command) = tokens.first() else {
            continue;
        };

        match command {
            "uci" => {
                println!("id name Minuette 1.0-dev");
                println!("id author analog hors");
                println!("uciok")
            }
            "ucinewgame" => {
                engine.reset();
            }
            "isready" => {
                println!("readyok");
            }
            "position" => {
                init_pos = match get_fen(&tokens) {
                    Some(fen) => fen.parse().expect("failed to parse fen"),
                    None => Board::startpos(),
                };

                current_pos = init_pos.clone();
                moves_played.clear();
                if let Some(tokens) = get_moves(&tokens) {
                    for token in tokens {
                        let mv = parse_uci_move(&current_pos, token)
                            .expect("failed to parse move");
                        
                        current_pos.play(mv);
                        moves_played.push(mv);
                    }
                }
            }
            "go" => {
                let wtime = get_clock_field(&tokens, "wtime").unwrap_or_default();
                let btime = get_clock_field(&tokens, "btime").unwrap_or_default();
                let winc = get_clock_field(&tokens, "winc").unwrap_or_default();
                let binc = get_clock_field(&tokens, "binc").unwrap_or_default();

                let (time, inc) = match current_pos.side_to_move() {
                    Color::White => (wtime, winc),
                    Color::Black => (btime, binc),
                };

                let limits = SearchLimits::PerGame {
                    clock: Duration::from_millis(time as u64),
                    increment: Duration::from_millis(inc as u64),
                };

                let mut best_move = None;
                engine.think(&init_pos, &moves_played, limits, &mut |info| {
                    best_move = Some(info.best_move);
                    println!(
                        "info depth {} nodes {} score cp {} time {} nps {} pv {}",
                        info.depth,
                        info.nodes,
                        info.eval,
                        info.time.as_millis(),
                        (info.nodes as f32 / info.time.as_secs_f32()) as u64,
                        display_uci_move(&current_pos, info.best_move),
                    );
                });

                let best_move = best_move.expect("missing best move?");
                println!("bestmove {}", display_uci_move(&current_pos, best_move));
            }
            "quit" => {
                break;
            }
            _ => {
                eprintln!("unknown uci command");
            }
        }
        flush_stdout();
    }
}

fn get_fen(tokens: &[&str]) -> Option<String> {
    let fen_index = tokens.iter().position(|&t| t == "fen")? + 1;
    Some(tokens[fen_index..fen_index + 6].join(" "))
}

fn get_moves<'s, 't>(tokens: &'s [&'t str]) -> Option<&'s [&'t str]> {
    let moves_index = tokens.iter().position(|&t| t == "moves")? + 1;
    Some(&tokens[moves_index..])
}

fn get_clock_field(tokens: &[&str], field: &str) -> Option<u32> {
    let field_index = tokens.iter().position(|&t| t == field)? + 1;
    Some(tokens[field_index].parse().expect("failed to parse int"))
}

fn flush_stdout() {
    std::io::stdout().flush().expect("failed to flush");
}
