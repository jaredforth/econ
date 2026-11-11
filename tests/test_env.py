import contextlib
import io
import os
import tempfile
import unittest
from unittest import mock

from thrift import cli


def run(argv, env):
    out = io.StringIO()
    with tempfile.TemporaryDirectory() as d, mock.patch.dict(os.environ, env):
        old = os.getcwd()
        os.chdir(d)
        try:
            if "LITELLM_MASTER_KEY" not in env:
                os.environ.pop("LITELLM_MASTER_KEY", None)
            with contextlib.redirect_stdout(out):
                cli.main(argv)
        finally:
            os.chdir(old)
    return out.getvalue()


class EnvTest(unittest.TestCase):
    def test_env(self):
        o = run(["env"], {"LITELLM_MASTER_KEY": "sk-x"})
        for s in ("export ANTHROPIC_BASE_URL=http://127.0.0.1:4000", "ANTHROPIC_AUTH_TOKEN=sk-x", "ANTHROPIC_MODEL=auto"):
            self.assertIn(s, o)

    def test_env_unset(self):
        with self.assertRaises(SystemExit) as e, contextlib.redirect_stderr(io.StringIO()):
            run(["env"], {})
        self.assertEqual(e.exception.code, 2)

    def test_bench_no_labels(self):
        with self.assertRaises(SystemExit) as e, contextlib.redirect_stderr(io.StringIO()):
            run(["bench", "m"], {})
        self.assertEqual(e.exception.code, 2)


if __name__ == "__main__":
    unittest.main()
