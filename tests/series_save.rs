use baseball_highlights_unfolded::{Action, Game, GloveReview, Refusal, Seat};
use std::fs;
use unfolded_record_replacement::save;

#[test]
fn a_checksummed_record_must_match_its_seed_and_legal_choices() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("series.json");
    let mut game = Game::open(&path).unwrap();
    let sf = Seat::SanFrancisco;
    let before = game.observe(sf);
    game.submit(before.submission(Action::ChooseOrder { first: sf }))
        .unwrap();
    let shopping = game.observe(sf);
    game.submit(shopping.submission(Action::Sign {
        slot: 5,
        card: shopping.offers[4].player.card,
    }))
    .unwrap();
    let previous_path = path.with_extension("json.previous");
    let previous = fs::read(&previous_path).unwrap();
    let (identity, payload) = save::unseal(&fs::read(&path).unwrap()).unwrap();

    let mut illegal_choice = payload.clone();
    illegal_choice["choices"][1]["seat"] = "new_york".into();

    // Both managers see the same public position even after this hidden change.
    let mut wrong_lineup = payload;
    wrong_lineup["position"]["clubs"][0]["lineup"]
        .as_array_mut()
        .unwrap()
        .swap(0, 1);

    for (changed, reason) in [
        (illegal_choice, "Cannot replay manager choice 2."),
        (
            wrong_lineup,
            "The saved position disagrees with its decisions.",
        ),
    ] {
        let resealed = save::seal(&identity, &changed);
        fs::write(&path, &resealed).unwrap();
        assert!(save::unseal(&resealed).is_ok(), "The checksum must pass");
        let error = Game::open(&path)
            .err()
            .expect("The game must refuse to open");
        assert!(matches!(error, Refusal::Save(detail) if detail.contains(reason)));
        assert_eq!(fs::read(&path).unwrap(), resealed);
        assert_eq!(fs::read(&previous_path).unwrap(), previous);
    }
}

#[test]
fn an_unsupported_format_stays_in_place_when_a_manager_starts_fresh_elsewhere() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("series.json");
    let fresh = temporary.path().join("fresh.json");
    let old = Game::open(&path).unwrap().observe(Seat::SanFrancisco);
    let (_, payload) = save::unseal(&fs::read(&path).unwrap()).unwrap();
    let unsupported = [
        ("game", "baseball-highlights-unfolded"),
        ("format", "unsupported-series-format"),
    ]
    .into_iter()
    .collect();
    let bytes = save::seal(&unsupported, &payload);
    fs::write(&path, &bytes).unwrap();

    let error = Game::open(&path).err().expect("The format must be refused");
    assert!(matches!(error, Refusal::Save(detail) if detail.contains("Unsupported save")));
    let restarted = Game::open(&fresh).unwrap().observe(Seat::SanFrancisco);
    assert_ne!(restarted.series, old.series);
    assert_eq!(fs::read(&path).unwrap(), bytes);
    assert!(!path.with_extension("json.previous").exists());
}

#[test]
fn the_launch_mode_must_match_the_saved_review() {
    let temporary = tempfile::tempdir().unwrap();
    let buy = temporary.path().join("buy.json");
    let glove = temporary.path().join("glove.json");
    Game::open(&buy).unwrap();
    Game::open_glove_review(&glove, GloveReview::Maris).unwrap();
    let saved = [fs::read(&buy).unwrap(), fs::read(&glove).unwrap()];

    for result in [
        Game::open(&glove),
        Game::open_glove_review(&buy, GloveReview::Maris),
        Game::open_glove_review(&glove, GloveReview::Lone),
    ] {
        let error = result.err().expect("The review mode must be refused");
        assert!(matches!(error, Refusal::Save(detail) if detail.contains("different review mode")));
    }
    assert_eq!([fs::read(&buy).unwrap(), fs::read(&glove).unwrap()], saved);
}
