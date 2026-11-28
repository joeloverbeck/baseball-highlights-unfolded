use baseball_highlights_unfolded::{Action, Decision, Game, GloveReview, Hit, RunnerSpeed, Seat};

#[test]
fn both_maris_glove_outcomes_are_visible_before_committing() {
    for box_number in [1, 2] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("series.json");
        let open = || Game::open_glove_review(&path, GloveReview::Maris).unwrap();
        let before = open().observe(Seat::SanFrancisco);
        let initial_bytes = std::fs::read(&path).unwrap();
        let mut game = open();
        let mut observer = open();
        let sf = game.observe(Seat::SanFrancisco);
        let ny = observer.observe(Seat::NewYork);
        assert_eq!(sf, before);
        assert_eq!(game.observe(Seat::NewYork), ny);
        let field = sf.field.as_ref().unwrap();
        assert_eq!(field, ny.field.as_ref().unwrap());
        assert_eq!(field.home, Seat::NewYork);
        assert_eq!(field.acting, Seat::SanFrancisco);
        assert_eq!(field.diamonds[0].score, 1);
        assert_eq!(field.diamonds[1].score, 2);
        assert_eq!(
            field.diamonds[1]
                .bases
                .map(|runner| runner.map(|pawn| pawn.speed)),
            [None, Some(RunnerSpeed::Average), None]
        );
        let play = field.play.unwrap();
        assert_eq!(play.defender.card.identity.to_string(), "ST-001");
        assert_eq!(play.batter.card.identity.to_string(), "FA-067");
        assert_eq!(ny.decision, None);
        let Some(Decision::Glove { targets }) = sf.decision.as_ref() else {
            panic!("Expected SF's Glove choice");
        };
        assert_eq!(
            targets
                .iter()
                .map(|preview| (preview.target.number, preview.cancelled))
                .collect::<Vec<_>>(),
            [(1, Hit::Single), (2, Hit::HomeRun)]
        );
        assert_eq!(targets[0].surviving[0].hit, Hit::HomeRun);
        assert_eq!(targets[0].diamonds[1].score, 4);
        assert_eq!(targets[0].diamonds[1].bases, [None; 3]);
        assert_eq!(targets[1].surviving[0].hit, Hit::Single);
        assert_eq!(targets[1].diamonds[1].score, 3);
        let bases = targets[1].diamonds[1].bases;
        assert_eq!(bases[0].unwrap().card.identity.to_string(), "FA-067");
        assert_eq!(&bases[1..], [None; 2]);

        let movement = |target: usize| {
            serde_json::to_value(&targets[target]).unwrap()["runner_advances"].clone()
        };
        let second = field.diamonds[1].bases[1].unwrap();
        let maris = bases[0].unwrap();
        assert_eq!(
            movement(0),
            serde_json::json!([
                {"runner": second, "from_base": 2, "to_base": null},
                {"runner": maris, "from_base": null, "to_base": null}
            ])
        );
        assert_eq!(
            movement(1),
            serde_json::json!([
                {"runner": second, "from_base": 2, "to_base": null},
                {"runner": maris, "from_base": null, "to_base": 1}
            ])
        );
        assert_eq!(game.observe(Seat::SanFrancisco), sf);
        assert_eq!(game.observe(Seat::NewYork), ny);
        assert_eq!(std::fs::read(&path).unwrap(), initial_bytes);

        let selected = &targets[box_number - 1];
        game.submit(sf.submission(Action::CancelHit {
            target: selected.target,
        }))
        .unwrap();
        assert!(observer.refresh().unwrap());
        let after = observer.observe(Seat::NewYork);
        assert_eq!(after, game.observe(Seat::NewYork));
        let reopened = open();
        assert_eq!(
            reopened.observe(Seat::SanFrancisco),
            game.observe(Seat::SanFrancisco)
        );
        assert_eq!(reopened.observe(Seat::NewYork), after);
        let field = after.field.unwrap();
        assert_eq!(field.diamonds, selected.diamonds);
        assert_eq!(field.threatened.len(), 1);
        assert_eq!(field.threatened[0].hit, Hit::Single);
        assert_eq!(
            field.threatened[0].target.card.identity.to_string(),
            "ST-001"
        );
        assert_eq!(field.batting, Seat::SanFrancisco);
    }
}

#[test]
fn a_pending_glove_survives_restart_and_refuses_wrong_seat_and_retries() {
    use baseball_highlights_unfolded::Refusal;

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("series.json");
    let game = Game::open_glove_review(&path, GloveReview::Maris).unwrap();
    let before = game.observe(Seat::SanFrancisco);
    let observer = game.observe(Seat::NewYork);
    let Some(Decision::Glove { targets }) = &before.decision else {
        panic!("Expected SF's Glove choice");
    };
    let target = targets[0].target;
    let action = Action::CancelHit { target };
    drop(game);

    let mut game = Game::open_glove_review(&path, GloveReview::Maris).unwrap();
    let mut second_window = Game::open_glove_review(&path, GloveReview::Maris).unwrap();
    assert_eq!(game.observe(Seat::SanFrancisco), before);
    assert_eq!(game.observe(Seat::NewYork), observer);
    let initial_bytes = std::fs::read(&path).unwrap();
    assert_eq!(
        game.submit(observer.submission(action)),
        Err(Refusal::WrongSeat)
    );
    let mut different_instance = target;
    different_instance.card.instance = different_instance.card.instance.saturating_add(1);
    assert_eq!(
        game.submit(before.submission(Action::CancelHit {
            target: different_instance
        })),
        Err(Refusal::HitChanged)
    );
    assert_eq!(std::fs::read(&path).unwrap(), initial_bytes);
    assert_eq!(game.observe(Seat::SanFrancisco), before);

    game.submit(before.submission(action)).unwrap();
    let after = game.observe(Seat::NewYork);
    let committed_bytes = std::fs::read(&path).unwrap();
    let different_action = Action::CancelHit {
        target: targets[1].target,
    };
    assert!(matches!(
        second_window.submit(before.submission(different_action)),
        Err(Refusal::Save(_))
    ));
    assert!(second_window.refresh().unwrap());
    assert_eq!(second_window.observe(Seat::NewYork), after);
    assert_eq!(
        game.submit(before.submission(action)),
        Err(Refusal::StaleDecision)
    );
    assert_eq!(
        game.submit(after.submission(action)),
        Err(Refusal::WrongDecision)
    );
    assert_eq!(std::fs::read(&path).unwrap(), committed_bytes);
    assert_eq!(
        Game::open_glove_review(&path, GloveReview::Maris)
            .unwrap()
            .observe(Seat::NewYork),
        after
    );
    let visible = serde_json::to_value(after).unwrap();
    assert!(visible["rosters"][0].get("hand").is_none());
    assert!(visible["rosters"][0].get("on_deck").is_none());
    assert!(visible.get("private_seed").is_none());
    let visible = serde_json::to_value(observer).unwrap();
    assert!(visible["rosters"][0].get("hand").is_none());
    assert!(visible["rosters"][0].get("on_deck").is_none());
    assert!(visible["decision"].is_null());
}

#[test]
fn zero_or_one_legal_target_resolves_without_asking_the_manager() {
    use baseball_highlights_unfolded::Event;

    for review in [GloveReview::Lone, GloveReview::NoTarget] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("series.json");
        let game = Game::open_glove_review(&path, review).unwrap();
        let view = game.observe(Seat::SanFrancisco);
        let field = view.field.as_ref().unwrap();
        assert_eq!(view.decision, None);
        assert_eq!(field.acting, Seat::NewYork);
        assert!(!field.glove_pending);
        assert_eq!(field.diamonds[1].score, 0);
        assert_eq!(field.diamonds[1].bases, [None; 3]);
        assert_eq!(field.threatened.len(), 1);
        assert_eq!(field.threatened[0].hit, Hit::Single);
        match (review, view.events.as_slice()) {
            (
                GloveReview::Lone,
                [
                    Event::GloveCancelled {
                        threat,
                        chosen: false,
                        ..
                    },
                    Event::ThreatsPlaced { .. },
                ],
            ) => {
                assert_eq!(threat.hit, Hit::Single);
                assert_eq!(threat.target.number, 1);
            }
            (GloveReview::NoTarget, [Event::ThreatsPlaced { .. }]) => {}
            _ => panic!("Unexpected public resolution: {:?}", view.events),
        }
        assert_eq!(
            Game::open_glove_review(&path, review)
                .unwrap()
                .observe(Seat::SanFrancisco),
            view
        );
    }
}

#[test]
fn rulebook_example_resolves_wiffles_surviving_boxes_before_mcgwire_threatens() {
    use baseball_highlights_unfolded::Event;

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("series.json");
    let mut game = Game::open_glove_review(&path, GloveReview::Rulebook).unwrap();
    let before = game.observe(Seat::SanFrancisco);
    let Some(Decision::Glove { targets }) = &before.decision else {
        panic!("Expected McGwire to choose a Wiffle hit box");
    };
    assert_eq!(
        targets
            .iter()
            .map(|preview| (preview.target.number, preview.cancelled))
            .collect::<Vec<_>>(),
        [(1, Hit::Single), (2, Hit::Single), (3, Hit::Single)]
    );
    game.submit(before.submission(Action::CancelHit {
        target: targets[1].target,
    }))
    .unwrap();

    // Bases Loaded v5 RB 11: the first surviving Slow batter reaches first;
    // the other sends that runner to second before settling on first.
    let resolved = game.observe(Seat::NewYork);
    let field = resolved.field.as_ref().unwrap();
    assert_eq!(field.diamonds[1].score, 0);
    assert_eq!(
        field.diamonds[1]
            .bases
            .map(|runner| runner.map(|pawn| pawn.speed)),
        [Some(RunnerSpeed::Slow), Some(RunnerSpeed::Slow), None]
    );
    let [
        Event::GloveCancelled {
            threat: cancelled,
            chosen: true,
            ..
        },
        Event::HitResolved {
            threat: first,
            runs: 0,
            ..
        },
        Event::HitResolved {
            threat: second,
            runs: 0,
            ..
        },
        Event::ThreatsPlaced { card, count: 2, .. },
    ] = resolved.events.as_slice()
    else {
        panic!("Expected Glove, two resolved singles, then McGwire's threats");
    };
    assert_eq!(
        [
            cancelled.target.number,
            first.target.number,
            second.target.number
        ],
        [2, 1, 3]
    );
    assert_eq!(card.identity.to_string(), "FA-082");
    assert_eq!(
        field
            .threatened
            .iter()
            .map(|threat| (threat.hit, threat.batter.speed))
            .collect::<Vec<_>>(),
        [
            (Hit::Single, RunnerSpeed::Fast),
            (Hit::Double, RunnerSpeed::Fast)
        ]
    );
    assert_eq!(
        Game::open_glove_review(&path, GloveReview::Rulebook)
            .unwrap()
            .observe(Seat::NewYork),
        resolved
    );
}
