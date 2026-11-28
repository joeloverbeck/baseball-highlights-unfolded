#[path = "support/reviewed_game_two.rs"]
mod reviewed_game_two;

use baseball_highlights_unfolded::{
    Action, Decision, Event, Game, GloveReview, Observation, Refusal, Seat, Submission,
};
use reviewed_game_two::{card as reviewed_card, hands as reviewed_hands};
use std::{fs, path::Path};
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

const SF: Seat = Seat::SanFrancisco;
const NY: Seat = Seat::NewYork;

fn submit(game: &mut Game, seat: Seat, action: Action) {
    game.submit(game.observe(seat).submission(action)).unwrap();
}

fn pending_foxx_demotion(path: &Path) -> (Game, [Observation; 2], Submission) {
    fs::write(path, include_bytes!("fixtures/expert-game-two-start.json")).unwrap();
    let mut game = Game::open(path).unwrap();
    submit(&mut game, SF, Action::ChooseOrder { first: SF });
    let foxx = game.observe(SF).offers[4].player.card;
    assert_eq!(foxx, reviewed_card("FA-065", 28));
    submit(
        &mut game,
        SF,
        Action::Sign {
            slot: 5,
            card: foxx,
        },
    );
    let pending = [game.observe(SF), game.observe(NY)];
    let Some(Decision::Demote { eligible }) = &pending[0].decision else {
        panic!("Foxx must leave a pending SF demotion");
    };
    assert_eq!(pending[1].decision, None);
    for view in &pending {
        assert_eq!(view.balances, [2, 7]);
        let replacement = view.offers[4].player.card;
        assert_eq!(replacement, reviewed_card("FA-067", 5));
        assert_eq!(
            view.events.last(),
            Some(&Event::Signed {
                seat: SF,
                card: foxx,
                slot: 5,
                paid: 4,
                replacement,
            }),
        );
    }
    let demoted = eligible
        .iter()
        .copied()
        .find(|card| card.identity.serial() == 4)
        .unwrap();
    let choice = pending[0].submission(Action::Demote { card: demoted });
    (game, pending, choice)
}

#[test]
fn recovery_restores_both_seats_at_foxxs_demotion_and_continues_independently() {
    let [sf_hand, ny_hand] = reviewed_hands();
    for active_bytes in [Some(b"damaged series record".as_slice()), None] {
        let temporary = tempfile::tempdir().unwrap();
        let original = temporary.path().join("original.json");
        let original_previous = original.with_extension("json.previous");
        let fresh = temporary.path().join("fresh.json");
        let (mut game, pending, demote) = pending_foxx_demotion(&original);
        let pending_bytes = fs::read(&original).unwrap();
        game.submit(demote.clone()).unwrap();
        assert_eq!(fs::read(&original_previous).unwrap(), pending_bytes);
        drop(game);
        if let Some(bytes) = active_bytes {
            fs::write(&original, bytes).unwrap();
        } else {
            fs::remove_file(&original).unwrap();
        }

        let recovered = Game::recover(&original, &fresh, None).unwrap();
        assert_eq!([recovered.observe(SF), recovered.observe(NY)], pending);
        drop(recovered);
        let mut continued = Game::open(&fresh).unwrap();
        assert_eq!([continued.observe(SF), continued.observe(NY)], pending);
        continued.submit(demote.clone()).unwrap();
        assert_eq!(continued.submit(demote), Err(Refusal::StaleDecision));
        let reopened = Game::open(&fresh).unwrap();
        assert_eq!(reopened.observe(SF), continued.observe(SF));
        assert_eq!(reopened.observe(NY), continued.observe(NY));
        assert_eq!(reopened.observe(SF).rosters[0].minor_leagues, 1);

        submit(&mut continued, SF, Action::Pass);
        submit(
            &mut continued,
            NY,
            Action::Sign {
                slot: 5,
                card: reviewed_card("FA-067", 5),
            },
        );
        let Some(Decision::Demote { eligible }) = continued.observe(NY).decision else {
            panic!("Maris must leave a pending NY demotion");
        };
        let card = eligible
            .into_iter()
            .find(|card| card.identity.serial() == 56)
            .unwrap();
        submit(&mut continued, NY, Action::Demote { card });
        submit(&mut continued, NY, Action::Pass);
        submit(&mut continued, SF, Action::StartGame);
        let reopened = Game::open(&fresh).unwrap();
        let sf = reopened.observe(SF);
        let ny = reopened.observe(NY);
        assert_eq!(sf.series, pending[0].series);
        assert_eq!(sf, continued.observe(SF));
        assert_eq!(ny, continued.observe(NY));
        assert_eq!(sf.rosters[0].hand.as_deref(), Some(sf_hand.as_slice()));
        assert_eq!(ny.rosters[1].hand.as_deref(), Some(ny_hand.as_slice()));
        assert_eq!(sf.rosters[1].hand, None);
        assert_eq!(ny.rosters[0].hand, None);
        assert_eq!(fs::read(&original_previous).unwrap(), pending_bytes);
        if let Some(bytes) = active_bytes {
            assert_eq!(fs::read(&original).unwrap(), bytes);
        } else {
            assert!(!original.exists());
        }
    }
}

#[test]
fn recovery_refuses_a_previous_record_that_fails_current_admission() {
    let temporary = tempfile::tempdir().unwrap();
    let original = temporary.path().join("original.json");
    let previous = original.with_extension("json.previous");
    let fresh = temporary.path().join("fresh.json");
    let (mut game, _, demote) = pending_foxx_demotion(&original);
    game.submit(demote).unwrap();
    let active_bytes = fs::read(&original).unwrap();
    let previous_bytes = fs::read(&previous).unwrap();
    let (identity, mut position) = save::unseal(&previous_bytes).unwrap();
    position["position"]["balances"][0] = 6.into();
    let foreign = [
        ("game", "baseball-highlights-unfolded"),
        ("format", "unsupported-series-format"),
    ]
    .into_iter()
    .collect();

    for (bytes, review, reason) in [
        (b"damaged previous".to_vec(), None, "Invalid save"),
        (save::seal(&foreign, &position), None, "Unsupported save"),
        (
            save::seal(&identity, &position),
            None,
            "The saved position disagrees with its decisions.",
        ),
        (
            previous_bytes,
            Some(GloveReview::Maris),
            "different review mode",
        ),
    ] {
        fs::write(&previous, &bytes).unwrap();
        let error = Game::recover(&original, &fresh, review)
            .err()
            .expect("Recovery must refuse an unusable previous record");
        assert!(matches!(error, Refusal::Save(detail) if detail.contains(reason)));
        assert_eq!(fs::read(&original).unwrap(), active_bytes);
        assert_eq!(fs::read(&previous).unwrap(), bytes);
        assert!(!fresh.exists());
        assert!(!fresh.with_extension("json.previous").exists());
    }
}

#[test]
fn recovery_requires_a_destination_without_a_save_or_recovery_history() {
    let temporary = tempfile::tempdir().unwrap();
    let original = temporary.path().join("original.json");
    let previous = original.with_extension("json.previous");
    let occupied = temporary.path().join("occupied.json");
    let orphaned = temporary.path().join("orphaned.json");
    let orphaned_previous = orphaned.with_extension("json.previous");
    let (mut game, _, demote) = pending_foxx_demotion(&original);
    game.submit(demote).unwrap();
    Game::open(&occupied).unwrap();
    let occupied_bytes = fs::read(&occupied).unwrap();
    fs::write(&orphaned_previous, &occupied_bytes).unwrap();
    let active_bytes = fs::read(&original).unwrap();
    let previous_bytes = fs::read(&previous).unwrap();

    for (destination, reason) in [
        (&original, "distinct from both source records"),
        (&previous, "distinct from both source records"),
        (&occupied, "already occupied"),
        (&orphaned, "previous record exists without an active save"),
    ] {
        let error = Game::recover(&original, destination, None)
            .err()
            .expect("Recovery needs an unused path");
        assert!(matches!(error, Refusal::Save(detail) if detail.contains(reason)));
        assert_eq!(fs::read(&original).unwrap(), active_bytes);
        assert_eq!(fs::read(&previous).unwrap(), previous_bytes);
        assert_eq!(fs::read(&occupied).unwrap(), occupied_bytes);
        assert!(!occupied.with_extension("json.previous").exists());
        assert!(!orphaned.exists());
        assert_eq!(fs::read(&orphaned_previous).unwrap(), occupied_bytes);
        assert!(!previous.with_extension("previous.previous").exists());
    }
}

#[test]
fn an_unusable_previous_record_prevents_a_new_decision_from_being_saved() {
    let temporary = tempfile::tempdir().unwrap();
    let active = temporary.path().join("series.json");
    let previous = active.with_extension("json.previous");
    let (game, pending, demote) = pending_foxx_demotion(&active);
    drop(game);
    let active_bytes = fs::read(&active).unwrap();
    let invalid_previous = b"unusable previous save";
    fs::write(&previous, invalid_previous).unwrap();
    let mut game = Game::open(&active).unwrap();

    let error = game.submit(demote).expect_err("Rotation must fail");
    assert!(matches!(error, Refusal::Save(detail)
        if detail.contains("Could not save the decision") && detail.contains("Invalid save")));
    assert_eq!([game.observe(SF), game.observe(NY)], pending);
    assert_eq!(fs::read(&active).unwrap(), active_bytes);
    assert_eq!(fs::read(&previous).unwrap(), invalid_previous);
}

#[test]
fn recovery_does_not_use_a_missing_source_as_a_directory_or_sidecar() {
    for (original_name, destination_name) in [
        ("original.json", "original.json/recovery/fresh.json"),
        (
            "original.json",
            "new-directory/../original.json/recovery/fresh.json",
        ),
        ("fresh.lock", "fresh"),
        ("fresh.previous", "fresh"),
    ] {
        let temporary = tempfile::tempdir().unwrap();
        let original = temporary.path().join(original_name);
        let previous = unfolded_record_replacement::previous_path(&original);
        let destination = temporary.path().join(destination_name);
        let (mut game, _, demote) = pending_foxx_demotion(&original);
        game.submit(demote).unwrap();
        drop(game);
        let previous_bytes = fs::read(&previous).unwrap();
        fs::remove_file(&original).unwrap();

        let error = Game::recover(&original, &destination, None)
            .err()
            .expect("Recovery must leave the missing source path free");
        assert!(
            matches!(error, Refusal::Save(detail) if detail.contains("distinct from both source records"))
        );
        assert!(!original.exists());
        assert!(!destination.exists());
        assert_eq!(fs::read(&previous).unwrap(), previous_bytes);
    }
}

#[cfg(unix)]
#[test]
fn recovery_refuses_a_destination_lock_link_that_could_create_the_missing_source() {
    let temporary = tempfile::tempdir().unwrap();
    let original = temporary.path().join("original.json");
    let previous = original.with_extension("json.previous");
    let fresh = temporary.path().join("fresh.json");
    let lock = fresh.with_extension("json.lock");
    let (mut game, _, demote) = pending_foxx_demotion(&original);
    game.submit(demote).unwrap();
    let previous_bytes = fs::read(&previous).unwrap();
    fs::remove_file(&original).unwrap();
    std::os::unix::fs::symlink("original.json", &lock).unwrap();

    for destination in [&fresh, &temporary.path().join("new/../fresh.json")] {
        let error = Game::recover(&original, destination, None)
            .err()
            .expect("A lock link must leave the original missing");
        assert!(matches!(error, Refusal::Save(detail) if detail.contains("destination lock")));
        assert!(!original.exists());
        assert!(!fresh.exists());
        assert!(!temporary.path().join("new").exists());
        assert_eq!(fs::read(&previous).unwrap(), previous_bytes);
        assert_eq!(fs::read_link(&lock).unwrap(), Path::new("original.json"));
    }
}

#[test]
fn recovery_refuses_directory_spellings_before_opening_a_lock() {
    let temporary = tempfile::tempdir().unwrap();
    let directory = temporary.path().join("destination/");
    fs::create_dir(&directory).unwrap();
    let original = directory.join(".lock");
    let previous = unfolded_record_replacement::previous_path(&original);
    let (mut game, _, demote) = pending_foxx_demotion(&original);
    game.submit(demote).unwrap();
    let previous_bytes = fs::read(&previous).unwrap();
    fs::remove_file(&original).unwrap();

    let error = Game::recover(&original, &directory, None)
        .err()
        .expect("Recovery needs a file destination");
    assert!(!original.exists(), "Refusal must leave the source absent");
    assert!(matches!(error, Refusal::Save(detail) if detail.contains("destination file path")));
    assert_eq!(fs::read(&previous).unwrap(), previous_bytes);
}
