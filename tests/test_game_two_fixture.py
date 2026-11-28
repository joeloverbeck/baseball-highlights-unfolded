"""Keep the documented game-two starting position bound to its fixture."""
import hashlib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = "tests/fixtures/expert-game-two-start.json"
SPEC = "docs/specs/two-expert-games.md"


class GameTwoFixture(unittest.TestCase):
    def test_current_scratch_save_matches_documented_checksum(self):
        actual = hashlib.sha256((ROOT / FIXTURE).read_bytes()).hexdigest()
        row_prefix = f"| [Current scratch save](../../{FIXTURE}) |"
        documented = [
            row.split("|")[2].strip()
            for row in (ROOT / SPEC).read_text(encoding="utf-8").splitlines()
            if row.startswith(row_prefix)
        ]
        self.assertEqual(
            documented,
            [f"`{actual}`"],
            f"Record the current SHA-256 for {FIXTURE} in {SPEC}.",
        )
