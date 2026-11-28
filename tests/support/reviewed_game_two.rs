use baseball_highlights_unfolded::Card;

pub(super) fn card(identity: &str, instance: u8) -> Card {
    Card {
        identity: identity.parse().unwrap(),
        instance,
    }
}

// SF and NY draw orders after the reviewed signings in the issue #8 fixture at 2f4a6cfee120.
pub(super) fn hands() -> [[Card; 6]; 2] {
    [
        [
            card("FA-065", 28),
            card("ST-013", 98),
            card("ST-009", 104),
            card("ST-003", 91),
            card("ST-012", 14),
            card("ST-006", 19),
        ],
        [
            card("FA-067", 5),
            card("ST-054", 53),
            card("ST-049", 20),
            card("ST-048", 18),
            card("ST-046", 25),
            card("ST-052", 12),
        ],
    ]
}
