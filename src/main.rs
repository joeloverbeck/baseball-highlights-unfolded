mod tabletop;

use baseball_highlights_unfolded::{Game, Seat, assets::Collection};
use std::{env, fs, path::PathBuf, process::ExitCode};

const USAGE: &str =
    "Usage: baseball-highlights-unfolded [--assets DIRECTORY] [--save FILE] [--agent-seat sf|ny]";

fn run() -> Result<(), String> {
    let mut assets = None;
    let mut save = None;
    let mut agent = None;
    let mut args = env::args_os().skip(1);
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--assets") => assets = Some(PathBuf::from(args.next().ok_or(USAGE)?)),
            Some("--save") => save = Some(PathBuf::from(args.next().ok_or(USAGE)?)),
            Some("--agent-seat") => {
                agent = Some(
                    match args.next().as_deref().and_then(|seat| seat.to_str()) {
                        Some("sf") => Seat::SanFrancisco,
                        Some("ny") => Seat::NewYork,
                        _ => return Err(USAGE.into()),
                    },
                );
            }
            Some("--help" | "-h") => {
                println!("{USAGE}");
                return Ok(());
            }
            _ => return Err(USAGE.into()),
        }
    }
    let home = || {
        env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or("Set HOME or supply --assets and --save.")
    };
    let assets = match assets {
        Some(path) => path,
        None => home()?.join("Pictures/Baseball Highlights: 2045 Resources"),
    };
    let save = match save {
        Some(path) => path,
        None => {
            let state = match env::var_os("XDG_STATE_HOME")
                .map(PathBuf::from)
                .filter(|p| p.is_absolute())
            {
                Some(path) => path,
                None => home()?.join(".local/state"),
            };
            state.join("baseball-highlights-unfolded/expert-buy-round.json")
        }
    };
    let collection = Collection::open(&assets)?;
    let art = tabletop::Art::load(&collection)?;
    if let Some(parent) = save.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
    }
    let game = Game::open(save).map_err(|error| error.to_string())?;
    if let Some(seat) = agent {
        tabletop::serve(game, seat, art)
    } else {
        tabletop::run(game, art)
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
