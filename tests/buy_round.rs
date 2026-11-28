use baseball_highlights_unfolded::{
    Action, BuyRoundFixture, Decision, Event, Game, Refusal, Seat, Submission,
};
use std::fs;

fn choose_order(game: &mut Game, chooser: Seat, first: Seat) {
    let view = game.observe(chooser);
    game.submit(view.submission(Action::ChooseOrder { first }))
        .unwrap();
}

#[test]
fn revenue_chooses_who_sets_purchase_order() {
    let temporary = tempfile::tempdir().unwrap();
    let game = Game::open(temporary.path().join("series.json")).unwrap();
    let sf = game.observe(Seat::SanFrancisco);
    let ny = game.observe(Seat::NewYork);
    assert_eq!(sf.order_chooser, Seat::SanFrancisco);
    assert_eq!(sf.acting_manager, Some(Seat::SanFrancisco));
    assert_eq!(sf.buyer, None);
    assert_eq!(sf.decision, Some(Decision::ChooseOrder));
    assert_eq!(ny.decision, None);
    assert_eq!(ny.revenue, sf.revenue);
    assert_eq!(sf.revenue.each_ref().map(|revenue| revenue.total), [6, 7]);
    assert_eq!(
        sf.revenue[0]
            .contributions
            .iter()
            .map(|entry| { (entry.card.identity.serial(), entry.amount) })
            .collect::<Vec<_>>(),
        [(2, 2), (4, 2), (5, 1), (8, 1), (11, 0), (14, 0)]
    );
    assert_eq!(
        sf.revenue[1]
            .contributions
            .iter()
            .map(|entry| (entry.card.identity.serial(), entry.amount))
            .collect::<Vec<_>>(),
        [(50, 1), (56, 1), (57, 1), (58, 2), (59, 0), (60, 2)]
    );
}

#[test]
fn san_francisco_can_offer_new_york_the_first_purchase() {
    let temporary = tempfile::tempdir().unwrap();
    let mut game = Game::open(temporary.path().join("series.json")).unwrap();
    let sf = game.observe(Seat::SanFrancisco);
    game.submit(sf.submission(Action::ChooseOrder {
        first: Seat::NewYork,
    }))
    .unwrap();
    assert!(matches!(
        game.observe(Seat::NewYork).decision,
        Some(Decision::Sign { .. })
    ));
    assert_eq!(game.observe(Seat::SanFrancisco).decision, None);

    let ny = game.observe(Seat::NewYork);
    assert_eq!(ny.balance, 7);
    assert!(matches!(&ny.decision, Some(Decision::Sign { affordable }) if !affordable.is_empty()));
    game.submit(ny.submission(Action::Pass)).unwrap();
    let sf = game.observe(Seat::SanFrancisco);
    assert_eq!(sf.acting_manager, Some(Seat::SanFrancisco));
    assert_eq!(sf.balances, [6, 0]);
    assert_eq!(sf.offers, game.observe(Seat::NewYork).offers);
    assert_eq!(
        sf.events.last(),
        Some(&Event::Passed {
            seat: Seat::NewYork,
            unspent: 7
        })
    );
}

#[test]
fn the_mini_game_loser_chooses_when_revenue_ties() {
    let temporary = tempfile::tempdir().unwrap();
    for first in [Seat::SanFrancisco, Seat::NewYork] {
        let path = temporary.path().join(first.abbreviation());
        let mut game = Game::open_buy_round_fixture(&path, BuyRoundFixture::RevenueTie).unwrap();
        let ny = game.observe(Seat::NewYork);
        let sf = game.observe(Seat::SanFrancisco);
        assert_eq!(ny.revenue.each_ref().map(|revenue| revenue.total), [6, 6]);
        assert_eq!(ny.revenue, sf.revenue);
        assert_eq!(ny.acting_manager, Some(Seat::NewYork));
        assert_eq!(sf.acting_manager, ny.acting_manager);
        assert_eq!(ny.decision, Some(Decision::ChooseOrder));
        assert_eq!(sf.decision, None);
        let choose = Action::ChooseOrder { first };
        let before = fs::read(&path).unwrap();
        assert_eq!(game.submit(sf.submission(choose)), Err(Refusal::WrongSeat));
        assert_eq!(fs::read(&path).unwrap(), before);
        game.submit(ny.submission(choose)).unwrap();
        let reopened = Game::open(&path).unwrap();
        for seat in [Seat::SanFrancisco, Seat::NewYork] {
            let view = reopened.observe(seat);
            assert_eq!(view.buyer, Some(first));
            assert_eq!(view.first_buyer, Some(first));
            assert_eq!(
                view.events.last(),
                Some(&Event::OrderChosen {
                    chooser: Seat::NewYork,
                    first,
                })
            );
        }
    }
}

#[test]
fn signing_foxx_refills_one_slot_and_resumes_the_demotion() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("series.json");
    let mut game = Game::open(&path).unwrap();
    choose_order(&mut game, Seat::SanFrancisco, Seat::SanFrancisco);
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
    let choices = stored["choices"]
        .as_array()
        .expect("recorded manager choices");
    assert_eq!(choices.len(), 2);
    assert_eq!(choices[1]["seat"], "san_francisco");
    assert_eq!(
        choices[1]["action"],
        serde_json::to_value(purchase.action).unwrap()
    );
    let [
        Event::OrderChosen {
            chooser: Seat::SanFrancisco,
            first: Seat::SanFrancisco,
        },
        Event::Signed {
            seat: Seat::SanFrancisco,
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
    assert_eq!(game.observe(Seat::NewYork), ny_after);
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
    assert!(
        matches!(done.events.last(), Some(Event::Demoted { seat: Seat::SanFrancisco, card }) if *card == eligible[1])
    );
    assert_eq!(done.events.len(), 3);
    assert_eq!(
        done.decision,
        Some(Decision::Sign {
            affordable: Vec::new()
        })
    );
    assert_eq!(game.submit(demote), Err(Refusal::StaleDecision));

    game.submit(done.submission(Action::Pass)).unwrap();
    assert!(opponent.refresh().unwrap());
    let before_ny = opponent.observe(Seat::NewYork);
    assert_eq!(before_ny.balances, [0, 7]);
    assert_eq!(before_ny.acting_manager, Some(Seat::NewYork));
    let maris = before_ny.offers[4].player.card;
    opponent
        .submit(before_ny.submission(Action::Sign {
            slot: 5,
            card: maris,
        }))
        .unwrap();
    let pending = opponent.observe(Seat::NewYork);
    assert!(game.refresh().unwrap());
    let sf_observes = game.observe(Seat::SanFrancisco);
    assert_eq!(sf_observes.offers, pending.offers);
    assert_eq!(sf_observes.events, pending.events);
    assert_eq!(sf_observes.acting_manager, Some(Seat::NewYork));
    assert_eq!(sf_observes.decision, None);
    assert_eq!(pending.offers[4].player.name, "Babe Bench");
    assert_eq!(pending.offers[4].cost, 5);
    assert_eq!(pending.balance, 0);
    let ny_demotion = pending.rosters[1].in_play[1].card;
    opponent
        .submit(pending.submission(Action::Demote { card: ny_demotion }))
        .unwrap();
    let after_demotion = opponent.observe(Seat::NewYork);
    opponent
        .submit(after_demotion.submission(Action::Pass))
        .unwrap();
    let complete = opponent.observe(Seat::NewYork);
    assert_eq!(complete.balances, [0, 0]);
    assert_eq!(complete.acting_manager, Some(Seat::SanFrancisco));
    assert_eq!(complete.buyer, None);
    assert_eq!(complete.decision, None);
    for roster in &complete.rosters {
        assert_eq!(roster.active, 15);
        assert_eq!(roster.minor_leagues, 1);
        assert!(roster.in_play.is_empty());
    }
    assert_eq!(
        complete.rosters.each_ref().map(|roster| roster
            .dugout
            .iter()
            .map(|player| player.card.identity.serial())
            .collect::<Vec<_>>()),
        [vec![2, 5, 8, 11, 14], vec![50, 57, 58, 59, 60]]
    );
    assert!(game.refresh().unwrap());
    assert_eq!(game.observe(Seat::SanFrancisco).events, complete.events);
}

#[test]
fn the_same_buyer_can_sign_the_refill_after_resuming() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("series.json");
    let mut game = Game::open_buy_round_fixture(&path, BuyRoundFixture::RepeatPurchase).unwrap();
    choose_order(&mut game, Seat::SanFrancisco, Seat::NewYork);
    let mut sf = Game::open(&path).unwrap();
    let ny = game.observe(Seat::NewYork);
    assert_eq!(ny.balance, 11);
    let foxx = ny.offers[4].player.card;
    let purchase = ny.submission(Action::Sign {
        slot: 5,
        card: foxx,
    });
    game.submit(purchase.clone()).unwrap();
    let after_first = game.observe(Seat::NewYork);
    assert_eq!(after_first.balance, 7);
    let first_demotion = after_first.rosters[1].in_play[0].card;
    game.submit(after_first.submission(Action::Demote {
        card: first_demotion,
    }))
    .unwrap();
    let between = game.observe(Seat::NewYork);
    drop(game);
    let mut game = Game::open(&path).unwrap();
    assert_eq!(game.observe(Seat::NewYork), between);
    assert!(sf.refresh().unwrap());
    let waiting = sf.observe(Seat::SanFrancisco);
    assert_eq!(waiting.offers, between.offers);
    assert_eq!(waiting.balances, between.balances);
    assert_eq!(waiting.acting_manager, between.acting_manager);
    assert_eq!(waiting.events, between.events);
    assert_eq!(waiting.decision, None);
    assert_eq!(game.submit(purchase.clone()), Err(Refusal::StaleDecision));
    let stale_card = Submission {
        revision: between.revision,
        ..purchase
    };
    assert_eq!(game.submit(stale_card), Err(Refusal::OfferChanged));
    let maris = between.offers[4].player.card;
    assert_eq!(maris.identity.serial(), 67);
    game.submit(between.submission(Action::Sign {
        slot: 5,
        card: maris,
    }))
    .unwrap();
    let after_second = game.observe(Seat::NewYork);
    assert_eq!(after_second.balance, 0);
    assert_eq!(after_second.offers[4].player.card.identity.serial(), 68);
    assert_eq!(after_second.rosters[1].lineup, 11);
    assert_eq!(
        game.submit(after_second.submission(Action::Demote {
            card: first_demotion
        })),
        Err(Refusal::Ineligible)
    );
    let second_demotion = after_second.rosters[1].in_play[0].card;
    game.submit(after_second.submission(Action::Demote {
        card: second_demotion,
    }))
    .unwrap();
    let done = game.observe(Seat::NewYork);
    let purchases = done
        .events
        .iter()
        .filter_map(|event| match event {
            Event::Signed {
                seat: Seat::NewYork,
                card,
                paid,
                replacement,
                slot: 5,
            } => Some((card.identity.serial(), *paid, replacement.identity.serial())),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(purchases, [(65, 4, 67), (67, 7, 68)]);
    assert_eq!(done.rosters[1].active, 15);
    assert_eq!(done.rosters[1].minor_leagues, 2);
    assert!(sf.refresh().unwrap());
    let other = sf.observe(Seat::SanFrancisco);
    assert_eq!(other.events, done.events);
    assert_eq!(other.offers, done.offers);
    assert_eq!(other.balances, done.balances);
    assert_eq!(other.acting_manager, done.acting_manager);
    assert_eq!(other.decision, None);
    assert_eq!(other.rosters[1].hand, None);
    assert_eq!(other.rosters[1].on_deck, None);

    game.submit(done.submission(Action::Pass)).unwrap();
    let sf = game.observe(Seat::SanFrancisco);
    game.submit(sf.submission(Action::Pass)).unwrap();
    let sf = game.observe(Seat::SanFrancisco);
    game.submit(sf.submission(Action::StartGame)).unwrap();
    let ny = Game::open(&path).unwrap().observe(Seat::NewYork);
    assert_eq!(&ny.rosters[1].hand.as_ref().unwrap()[..2], &[maris, foxx]);
}

#[test]
fn refused_choices_leave_the_observation_and_exact_save_unchanged() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("series.json");
    let mut game = Game::open(&path).unwrap();
    choose_order(&mut game, Seat::SanFrancisco, Seat::SanFrancisco);
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
        choose_order(&mut game, Seat::SanFrancisco, Seat::SanFrancisco);
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
    choose_order(&mut first, Seat::SanFrancisco, Seat::SanFrancisco);
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

#[test]
fn fresh_series_have_private_orders_and_the_same_reviewed_offers() {
    let temporary = tempfile::tempdir().unwrap();
    let series = |name| {
        let path = temporary.path().join(name);
        let game = Game::open(&path).unwrap();
        let view = game.observe(Seat::SanFrancisco);
        let (_, payload) =
            unfolded_record_replacement::save::unseal(&fs::read(path).unwrap()).unwrap();
        (view, payload)
    };
    let (first, first_record) = series("first.json");
    let (second, second_record) = series("second.json");
    assert_ne!(first_record["private_seed"], second_record["private_seed"]);
    let order = |position: &serde_json::Value| {
        position["position"]["free_agent_deck"]
            .as_array()
            .unwrap()
            .iter()
            .map(|card| card["identity"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>()
    };
    assert_ne!(order(&first_record), order(&second_record));
    for (left, right) in first.offers.iter().zip(&second.offers) {
        assert_eq!(left.player.card.identity, right.player.card.identity);
        assert_eq!(left.cost, right.cost);
    }
    let visible = serde_json::to_value(first).unwrap();
    assert!(visible.get("private_seed").is_none());
    assert_eq!(visible["free_agent_deck"], 54);
}

#[test]
fn starter_proxies_keep_distinct_logical_and_physical_identities() {
    use baseball_highlights_unfolded::{Card, CardId, Player};

    for (id, face) in [
        (3, 4),
        (9, 10),
        (19, 18),
        (24, 25),
        (34, 33),
        (39, 40),
        (48, 49),
        (55, 54),
    ] {
        let starter = |number| format!("ST-{number:03}").parse::<CardId>().unwrap();
        let logical = starter(id);
        let printed = starter(face);
        assert_ne!(logical, printed);
        assert_eq!(logical.printed_face(), printed);
        assert_eq!(
            serde_json::to_value(logical).unwrap(),
            format!("ST-{id:03}")
        );
    }

    let proxy = Player::describe(Card {
        identity: "ST-003".parse().unwrap(),
        instance: 1,
    })
    .unwrap();
    let donor = Player::describe(Card {
        identity: "ST-004".parse().unwrap(),
        instance: 2,
    })
    .unwrap();
    assert_eq!(proxy.printed_face, donor.printed_face);
    assert_eq!(proxy.revenue, donor.revenue);
    assert_ne!(proxy.card, donor.card);
}
