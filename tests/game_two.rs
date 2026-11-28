use baseball_highlights_unfolded::{Action, Card, Decision, Event, Game, Hit, Refusal, Seat};
use std::fs;

const SF: Seat = Seat::SanFrancisco;
const NY: Seat = Seat::NewYork;

fn submit(game: &mut Game, seat: Seat, action: Action) {
    game.submit(game.observe(seat).submission(action)).unwrap();
}

fn buy(game: &mut Game, seat: Seat, serial: u8, demotion: u8) -> (Card, Card) {
    let view = game.observe(seat);
    let signed = view.offers[4].player.card;
    assert_eq!(signed.identity.serial(), serial);
    submit(
        game,
        seat,
        Action::Sign {
            slot: 5,
            card: signed,
        },
    );
    let Some(Decision::Demote { eligible }) = game.observe(seat).decision else {
        panic!("The acquisition requires a demotion");
    };
    let demoted = eligible
        .into_iter()
        .find(|card| card.identity.serial() == demotion)
        .unwrap();
    submit(game, seat, Action::Demote { card: demoted });
    submit(game, seat, Action::Pass);
    (signed, demoted)
}

#[test]
fn game_two_draws_the_actual_lineups_after_both_purchases() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("series.json");
    fs::write(&path, include_bytes!("fixtures/expert-game-two-start.json")).unwrap();
    let mut game = Game::open(&path).unwrap();
    // Known seed and reviewed signings put these physical cards on top in draw order.
    let expected = [
        [
            ("FA-065", 28),
            ("ST-013", 98),
            ("ST-009", 104),
            ("ST-003", 91),
            ("ST-012", 14),
            ("ST-006", 19),
        ],
        [
            ("FA-067", 5),
            ("ST-054", 53),
            ("ST-049", 20),
            ("ST-048", 18),
            ("ST-046", 25),
            ("ST-052", 12),
        ],
    ];

    submit(&mut game, SF, Action::ChooseOrder { first: SF });
    let (foxx, sf_demoted) = buy(&mut game, SF, 65, 4);
    let (maris, ny_demoted) = buy(&mut game, NY, 67, 56);
    let closed = game.observe(SF);
    assert_eq!(closed.balances, [0, 0]);
    assert_eq!(closed.decision, Some(Decision::StartGame));
    for (index, demoted) in [sf_demoted, ny_demoted].into_iter().enumerate() {
        assert_eq!(closed.rosters[index].active, 15);
        assert_eq!(closed.rosters[index].dugout.len(), 5);
        assert_eq!(closed.rosters[index].lineup, 10);
        assert_eq!(closed.rosters[index].minor_leagues, 1);
        assert!(closed.events.contains(&Event::Demoted {
            seat: [SF, NY][index],
            card: demoted,
        }));
    }

    let ny_closed = game.observe(NY);
    drop(game);
    let mut game = Game::open(&path).unwrap();
    let mut observer = Game::open(&path).unwrap();
    assert_eq!(game.observe(SF), closed);
    assert_eq!(observer.observe(NY), ny_closed);
    let wrong_seat = game.observe(NY).submission(Action::StartGame);
    assert_eq!(game.submit(wrong_seat), Err(Refusal::WrongSeat));
    submit(&mut game, SF, Action::StartGame);
    assert!(observer.refresh().unwrap());
    let sf = game.observe(SF);
    let ny = observer.observe(NY);
    assert_eq!(sf.series, closed.series);
    let standing = sf.standing.unwrap();
    assert_eq!(standing.game, 2);
    assert_eq!(standing.home, NY);
    assert_eq!(standing.wins, [0, 1]);
    assert_eq!(ny.standing, sf.standing);
    assert_eq!(sf.field, ny.field);
    assert_eq!(sf.acting_manager, Some(SF));
    for (index, view) in [sf, ny].into_iter().enumerate() {
        let hand = view.rosters[index].hand.as_ref().unwrap();
        let expected_hand = expected[index].map(|(identity, instance)| Card {
            identity: identity.parse().unwrap(),
            instance,
        });
        assert_eq!(hand, &expected_hand);
        assert_eq!(view.rosters[1 - index].hand, None);
        assert_eq!(view.rosters[index].active, 15);
        assert_eq!(view.rosters[index].lineup, 4);
    }

    let sf = game.observe(SF);
    let invalid = sf.submission(Action::PlayCard { card: sf_demoted });
    let before = fs::read(&path).unwrap();
    assert_eq!(game.submit(invalid), Err(Refusal::CardNotInHand));
    assert_eq!(fs::read(&path).unwrap(), before);
    let first = sf.rosters[0]
        .hand
        .as_ref()
        .unwrap()
        .iter()
        .copied()
        .find(|card| card.identity.serial() == 13)
        .unwrap();
    submit(&mut game, SF, Action::PlayCard { card: first });
    submit(&mut game, NY, Action::PlayCard { card: maris });
    submit(&mut game, SF, Action::PlayCard { card: foxx });
    let sf = game.observe(SF);
    let Some(Decision::Glove { targets }) = &sf.decision else {
        panic!("Foxx must answer Maris's two threats");
    };
    assert_eq!(targets.len(), 2);
    assert_eq!(targets[0].target.card, maris);
    assert_eq!(targets[0].cancelled, Hit::Single);
    assert_eq!(targets[0].diamonds[1].score, 1);
    assert_eq!(targets[0].diamonds[1].bases, [None; 3]);
    assert_eq!(targets[1].target.card, maris);
    assert_eq!(targets[1].cancelled, Hit::HomeRun);
    assert_eq!(targets[1].diamonds[1].score, 0);
    assert_eq!(targets[1].diamonds[1].bases[0].unwrap().card, maris);
    assert_eq!(sf.field.as_ref().unwrap().diamonds[1].bases, [None; 3]);
    assert!(observer.refresh().unwrap());
    let ny = observer.observe(NY);
    assert_eq!(ny.decision, None);
    assert_eq!(ny.field, sf.field);
    let commit = sf.submission(Action::CancelHit {
        target: targets[1].target,
    });
    assert_eq!(Game::open(&path).unwrap().observe(SF), sf);
    game.submit(commit.clone()).unwrap();
    assert_eq!(game.submit(commit), Err(Refusal::StaleDecision));
    assert!(observer.refresh().unwrap());
    let committed = game.observe(SF);
    assert_eq!(committed.field, observer.observe(NY).field);
    assert_eq!(committed.decision, None);
    let field = committed.field.unwrap();
    assert_eq!(field.diamonds, targets[1].diamonds);
    assert_eq!(field.threatened[0].target.card, foxx);
    assert_eq!(field.threatened[0].hit, Hit::Double);
    assert!(committed.events.contains(&Event::CardPlayed {
        seat: NY,
        card: maris
    }));
    let reopened = Game::open(&path).unwrap();
    assert_eq!(reopened.observe(SF), game.observe(SF));
    assert_eq!(reopened.observe(NY), observer.observe(NY));
}
