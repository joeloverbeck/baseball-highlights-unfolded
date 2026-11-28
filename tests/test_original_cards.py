"""Check the original-set admission contract against the public source ledger."""
import csv
import unittest
from collections import defaultdict
from pathlib import Path

LEDGER = Path(__file__).resolve().parents[1] / "docs/foundation/card-content.csv"
# Owner-approved gameplay equivalents and artwork donors, in original team order.
PROXIES = {
    "ST-003": "ST-004", "ST-009": "ST-010",
    "ST-019": "ST-018", "ST-024": "ST-025",
    "ST-034": "ST-033", "ST-039": "ST-040",
    "ST-048": "ST-049", "ST-055": "ST-054",
}
RECOVERED_ART = {
    "ST-019": "present:photograph",
    "ST-024": "present:photograph",
    "ST-034": "present:photograph",
    "ST-039": "present:photograph",
    "ST-055": "present:licensed-app",
}


class OriginalCardLedger(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.sources = defaultdict(list)
        with LEDGER.open(newline="") as ledger:
            for row in csv.DictReader(ledger):
                cls.sources[row["identity"]].append(row)

    def check_present(self, rows):
        self.assertTrue(rows, "missing source")
        for row in rows:
            self.assertEqual(row["art"], "present")
            self.assertTrue(row["tts_id"].isdecimal(), row)
            self.assertGreater(int(row["instances"]), 0)

    def test_four_starting_rosters(self):
        teams = ("San Francisco", "Los Angeles", "Boston", "New York")
        for team_index, team in enumerate(teams):
            first = 1 + 15 * team_index
            for serial in range(first, first + 15):
                identity = f"ST-{serial:03}"
                with self.subTest(team=team, card=identity):
                    rows = self.sources[identity]
                    if identity in PROXIES:
                        donor = PROXIES[identity]
                        art = RECOVERED_ART.get(identity, f"proxy:{donor}")
                        self.assertEqual([r["art"] for r in rows], [art])
                        self.assertEqual((rows[0]["tts_id"], rows[0]["instances"]), ("", ""))
                        self.check_present(self.sources[donor])
                    else:
                        # Several Workshop IDs can show the same printed starter.
                        self.check_present(rows)

    def test_sixty_original_free_agents(self):
        for serial in range(61, 121):
            identity = f"FA-{serial:03}"
            with self.subTest(card=identity):
                rows = self.sources[identity]
                self.assertEqual(len(rows), 1)
                self.check_present(rows)
                self.assertEqual(rows[0]["instances"], "1")

    def test_workshop_ids_identify_distinct_sources(self):
        seen = {}
        for identity, rows in self.sources.items():
            for row in rows:
                source = row["tts_id"]
                if source:
                    self.assertNotIn(source, seen, f"Workshop ID {source} reused for {identity}")
                    seen[source] = identity
