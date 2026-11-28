//! Interpret the marks on a displayed player face. The caller supplies any action.
use super::{
    Prints,
    controls::{INK, picture, row, text},
};
use baseball_highlights_unfolded::Player;
use bevy::prelude::*;
use serde::Serialize;

#[derive(Clone, Copy, Serialize)]
pub(super) struct Symbol {
    mark: &'static str,
    meaning: &'static str,
    #[serde(skip)]
    image: usize,
}

const SLOW: Symbol = Symbol {
    mark: "S",
    meaning: "Slow runner. A runner already on base advances the hit distance when the path is clear.",
    image: 0,
};
const AVERAGE: Symbol = Symbol {
    mark: "A",
    meaning: "Average runner. A runner already on base advances the hit distance. A single brings this runner home from second.",
    image: 1,
};
const FAST: Symbol = Symbol {
    mark: "F",
    meaning: "Fast runner. A runner already on base advances one extra base on a hit when the path is clear.",
    image: 2,
};
const PINCH_HIT: Symbol = Symbol {
    mark: "PH",
    meaning: "Pinch hit. Discard this card from your hand to play your On Deck card or the unseen top of your Lineup.",
    image: 3,
};

pub(super) fn symbols(player: Player) -> impl Iterator<Item = Symbol> {
    let speed = match player.speed {
        "Slow" => SLOW,
        "Fast" => FAST,
        _ => AVERAGE,
    };
    [Some(speed), player.pinch_hitter.then_some(PINCH_HIT)]
        .into_iter()
        .flatten()
}

pub(super) fn draw(parent: &mut ChildSpawnerCommands, prints: &Prints, player: Player, scale: f32) {
    for symbol in symbols(player) {
        parent
            .spawn(Node {
                align_items: AlignItems::Center,
                ..row(10. * scale)
            })
            .with_children(|line| {
                picture(
                    line,
                    prints.card_symbols[symbol.image].clone(),
                    34. * scale,
                    34. * scale,
                );
                text(line, symbol.meaning, 15. * scale, INK);
            });
    }
}
