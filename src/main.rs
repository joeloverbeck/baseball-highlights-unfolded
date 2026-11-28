mod tabletop;

use baseball_highlights_unfolded::{Game, GloveReview, Seat, assets::Collection};
use std::{env, path::PathBuf, process::ExitCode};

const USAGE: &str = "Usage: baseball-highlights-unfolded [--assets DIRECTORY] [--save FILE] [--recover-from ORIGINAL] [--review-glove] [--seat sf|ny | --agent-seat sf|ny]";

fn run() -> Result<(), String> {
    let mut assets = None;
    let mut save = None;
    let mut recover_from = None;
    let mut agent = None;
    let mut desktop_seat = None;
    let mut review_glove = false;
    let mut args = env::args_os().skip(1);
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--assets") => assets = Some(PathBuf::from(args.next().ok_or(USAGE)?)),
            Some("--save") => save = Some(PathBuf::from(args.next().ok_or(USAGE)?)),
            Some("--recover-from") if recover_from.is_none() => {
                recover_from = Some(PathBuf::from(
                    args.next()
                        .ok_or("--recover-from requires the original save path.")?,
                ));
            }
            Some("--review-glove") => review_glove = true,
            Some("--seat" | "--agent-seat") => {
                if desktop_seat.is_some() || agent.is_some() {
                    return Err(USAGE.into());
                }
                let seat = match args.next().as_deref().and_then(|seat| seat.to_str()) {
                    Some("sf") => Seat::SanFrancisco,
                    Some("ny") => Seat::NewYork,
                    _ => return Err(USAGE.into()),
                };
                if arg == "--seat" {
                    desktop_seat = Some(seat);
                } else {
                    agent = Some(seat);
                }
            }
            Some("--help" | "-h") => {
                println!("{USAGE}\nRecover ORIGINAL.previous into an unused --save FILE.");
                return Ok(());
            }
            _ => return Err(USAGE.into()),
        }
    }
    if recover_from.is_some() && save.is_none() {
        return Err("--recover-from requires --save FILE with an unused path.".into());
    }
    let home = || {
        env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or("Set HOME or supply --assets and --save.")
    };
    let assets = match assets {
        Some(path) => path,
        None => home()?.join("Pictures/Baseball Highlights: 2045 PhotoCraft"),
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
            let filename = if review_glove {
                "glove-review.json"
            } else {
                "expert-buy-round.json"
            };
            state.join("baseball-highlights-unfolded").join(filename)
        }
    };
    let collection = Collection::open(&assets)?;
    let requested_review = review_glove.then_some(GloveReview::Maris);
    let art = tabletop::Art::load(&collection, requested_review)?;
    let game = if let Some(original) = recover_from {
        Game::recover(&original, save, requested_review).map_err(|error| {
            format!(
                "Could not recover the previous committed record for {}. {error}",
                original.display()
            )
        })?
    } else {
        match requested_review {
            Some(fixture) => Game::open_glove_review(save, fixture),
            None => Game::open(save),
        }
        .map_err(|error| {
            format!("{error} To start a fresh series, use --save FILE with an unused path.")
        })?
    };
    if let Some(seat) = agent {
        tabletop::serve(game, seat, art)
    } else {
        tabletop::run(game, desktop_seat.unwrap_or(Seat::SanFrancisco), art)
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
