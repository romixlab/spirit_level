"""The card script's behaviour in a fake DOM, run by node (crates/sl_book_refs/tests/refs.test.mjs)"""

import shutil
import subprocess
from pathlib import Path

import pytest

TESTS = Path(__file__).resolve().parents[3] / "crates" / "sl_book_refs" / "tests"


@pytest.mark.skipif(shutil.which("node") is None, reason="node is not installed")
def test_the_card_script():
    run = subprocess.run(["node", "--test", str(TESTS)], capture_output=True, text=True)
    assert run.returncode == 0, run.stdout + run.stderr
