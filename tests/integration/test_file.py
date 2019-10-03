import json
from   pathlib import Path
import subprocess
import tempfile

IR_EXE = Path(__file__).parents[2] / "target/debug/ir"
TEST_EXE = Path(__file__).parent / "test.py"


def run(spec):
    with tempfile.NamedTemporaryFile(mode="w+") as tmp_file:
        json.dump(spec, tmp_file)
        tmp_file.flush()
        res = subprocess.run(
            [str(IR_EXE), tmp_file.name],
            stdout=subprocess.PIPE,
            check=True,
        )
    return json.loads(res.stdout)


def test_stdout_stderr(tmp_path):
    stdout_path = tmp_path / "stdout"
    stderr_path = tmp_path / "stderr"
    result = run({
        "argv": [str(TEST_EXE), "--exit", "42"],
        "fds": [
            {"fd": 1, "file": {"path": str(stdout_path)}},
            {"fd": 2, "file": {"path": str(stderr_path)}},
        ]
    })
    assert result["status"] == 42 << 8  # FIXME
    assert stdout_path.read_text() == (
        "message 0 to stdout\n"
        "message 2 to stdout\n"
    )
    assert stderr_path.read_text() == (
        "message 1 to stderr\n"
    )

