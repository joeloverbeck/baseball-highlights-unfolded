use baseball_highlights_unfolded::{Action, Decision, Event, Game, Refusal, Seat, Submission};
use std::fs;

#[test]
fn signing_foxx_refills_one_slot_and_resumes_the_demotion() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("series.json");
    let mut game = Game::open(&path).unwrap();
    let before = game.observe(Seat::SanFrancisco);
    let mut opponent = Game::open(&path).unwrap();
    let ny_before = opponent.observe(Seat::NewYork);
    assert_eq!(before.balance, 6);
    assert_eq!(
        before
            .offers
            .each_ref()
            .map(|offer| offer.player.card.identity.serial()),
        [61, 62, 63, 64, 65, 66]
    );
    assert_eq!(before.offers[1].player.name, "Peewee Rizzuto");
    assert_eq!(before.offers[1].cost, 4);
    assert_eq!(before.offers[4].player.name, "Frankie Foxx");
    assert_eq!(before.offers[4].balance_after, Some(2));
    assert_eq!(before.offers[0].balance_after, None);
    assert_eq!(ny_before.decision, None);
    let foxx = before.offers[4].player.card;
    let purchase = before.submission(Action::Sign {
        slot: 5,
        card: foxx,
    });
    game.submit(purchase.clone()).unwrap();
    let after = game.observe(Seat::SanFrancisco);
    assert!(opponent.refresh().unwrap());
    let ny_after = opponent.observe(Seat::NewYork);
    assert_eq!(after.balance, 2);
    assert_eq!(after.offers[4].player.name, "Mickey Maris");
    for slot in [0, 1, 2, 3, 5] {
        assert_eq!(after.offers[slot].player, before.offers[slot].player);
    }
    assert_eq!(ny_after.offers, after.offers);
    assert_eq!(ny_after.events, after.events);
    assert_eq!(ny_after.decision, None);
    assert_eq!(after.rosters[0].lineup, 10);
    let (_, stored) = unfolded_record_replacement::save::unseal(&fs::read(&path).unwrap()).unwrap();
    let lineup = stored["clubs"][0]["lineup"].as_array().unwrap();
    assert_eq!(lineup.last(), Some(&serde_json::to_value(foxx).unwrap()));
    for id in ["ST-003", "ST-009"] {
        assert!(lineup.iter().any(|card| card["identity"] == id));
    }
    let [
        Event::Signed {
            paid: 4,
            card,
            slot: 5,
            replacement,
        },
    ] = after.events.as_slice()
    else {
        panic!("Expected one public signing and refill");
    };
    assert_eq!(*card, foxx);
    assert_eq!(*replacement, after.offers[4].player.card);
    let encoded = serde_json::to_string(&ny_after).unwrap();
    for hidden in ["FA-068", "FA-120", "ST-003", "ST-009", "ST-012"] {
        assert!(!encoded.contains(hidden), "{hidden} leaked");
    }
    assert_eq!(ny_after.rosters[0].hand.as_deref(), None);
    assert_eq!(ny_after.rosters[0].on_deck, None);
    let saved = fs::read(&path).unwrap();
    drop(game);

    let mut game = Game::open(&path).unwrap();
    assert_eq!(game.observe(Seat::SanFrancisco), after);
    assert_eq!(game.submit(purchase), Err(Refusal::StaleDecision));
    assert_eq!(fs::read(&path).unwrap(), saved);
    let Some(Decision::Demote { eligible }) = &after.decision else {
        panic!("Expected the required demotion");
    };
    assert_eq!(
        eligible
            .iter()
            .map(|card| card.identity.serial())
            .collect::<Vec<_>>(),
        [2, 4, 5, 8, 11, 14]
    );
    let demote = after.submission(Action::Demote { card: eligible[1] });
    game.submit(demote.clone()).unwrap();
    let done = game.observe(Seat::SanFrancisco);
    assert_eq!(done.rosters[0].active, 15);
    assert_eq!(done.rosters[0].minor_leagues, 1);
    assert_eq!(done.rosters[0].in_play.len(), 5);
    assert!(matches!(done.events.last(), Some(Event::Demoted { card }) if *card == eligible[1]));
    assert_eq!(done.events.len(), 2);
    assert_eq!(done.decision, None);
    assert_eq!(game.submit(demote), Err(Refusal::StaleDecision));
}

#[test]
fn refused_choices_leave_the_observation_and_exact_save_unchanged() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("series.json");
    let mut game = Game::open(&path).unwrap();
    let ready = game.observe(Seat::SanFrancisco);
    let cano = ready.submission(Action::Sign {
        slot: 1,
        card: ready.offers[0].player.card,
    });
    let foxx = ready.submission(Action::Sign {
        slot: 5,
        card: ready.offers[4].player.card,
    });
    let initial = fs::read(&path).unwrap();
    let wrong_seat = Submission {
        seat: Seat::NewYork,
        ..foxx.clone()
    };
    assert_eq!(game.submit(wrong_seat), Err(Refusal::WrongSeat));
    assert_eq!(game.submit(cano), Err(Refusal::Unaffordable));
    assert_eq!(game.observe(Seat::SanFrancisco), ready);
    assert_eq!(fs::read(&path).unwrap(), initial);
    game.submit(foxx.clone()).unwrap();
    let pending = game.observe(Seat::SanFrancisco);
    let eligible = pending.rosters[0].in_play[1].card;
    let other_team = pending.rosters[1].in_play[0].card;
    let alias = baseball_highlights_unfolded::Card {
        identity: "ST-003".parse().unwrap(),
        ..eligible
    };
    let baseline = fs::read(&path).unwrap();
    for card in [other_team, alias, ready.offers[4].player.card] {
        assert_eq!(
            game.submit(pending.submission(Action::Demote { card })),
            Err(Refusal::Ineligible)
        );
    }
    let duplicate = Submission {
        revision: pending.revision,
        ..foxx
    };
    assert_eq!(game.submit(duplicate), Err(Refusal::WrongDecision));
    assert_eq!(game.observe(Seat::SanFrancisco), pending);
    assert_eq!(fs::read(&path).unwrap(), baseline);
}

#[test]
fn every_in_play_player_can_be_the_one_demotion() {
    for index in 0..6 {
        let temporary = tempfile::tempdir().unwrap();
        let mut game = Game::open(temporary.path().join("series.json")).unwrap();
        let ready = game.observe(Seat::SanFrancisco);
        game.submit(ready.submission(Action::Sign {
            slot: 5,
            card: ready.offers[4].player.card,
        }))
        .unwrap();
        let pending = game.observe(Seat::SanFrancisco);
        let card = pending.rosters[0].in_play[index].card;
        game.submit(pending.submission(Action::Demote { card }))
            .unwrap();
        let finished = game.observe(Seat::SanFrancisco);
        assert_eq!(finished.rosters[0].active, 15);
        assert_eq!(finished.rosters[0].minor_leagues, 1);
        assert!(
            !finished.rosters[0]
                .in_play
                .iter()
                .any(|player| player.card == card)
        );
    }
}

#[test]
fn a_second_writer_cannot_replace_the_first_signing() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("series.json");
    let mut first = Game::open(&path).unwrap();
    let mut second = Game::open(&path).unwrap();
    let before = second.observe(Seat::SanFrancisco);
    first
        .submit(before.submission(Action::Sign {
            slot: 2,
            card: before.offers[1].player.card,
        }))
        .unwrap();
    let committed = fs::read(&path).unwrap();
    let loser = second.submit(before.submission(Action::Sign {
        slot: 5,
        card: before.offers[4].player.card,
    }));
    assert!(matches!(loser, Err(Refusal::Save(_))));
    assert_eq!(fs::read(&path).unwrap(), committed);
    assert_eq!(second.observe(Seat::SanFrancisco), before);
    assert!(second.refresh().unwrap());
    assert_eq!(
        second.observe(Seat::SanFrancisco),
        first.observe(Seat::SanFrancisco)
    );
}
