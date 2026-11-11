import contextlib
import io
import os
import tempfile
import unittest
from unittest import mock

from thrift import classify, cli

ENV = {"LITELLM_MASTER_KEY": "k", "THRIFT_SIMPLE": "ollama/x", "THRIFT_MODERATE": "ollama/y", "THRIFT_HARD": "ollama/z"}


def run(env, files=()):
    out = io.StringIO()
    with tempfile.TemporaryDirectory() as d:
        for f in files:
            open(os.path.join(d, f), "w").close()
        old = os.getcwd()
        os.chdir(d)
        try:
            with mock.patch.dict(os.environ, env, clear=True), contextlib.redirect_stdout(out):
                try:
                    cli.main(["doctor"])
                    code = 0
                except SystemExit as e:
                    code = e.code
        finally:
            os.chdir(old)
    return code, out.getvalue()


class DoctorTest(unittest.TestCase):
    def test_fail(self):
        code, out = run({"OLLAMA_HOST": "http://127.0.0.1:9"})
        self.assertEqual(code, 1)
        self.assertIn("ollama serve", out)

    def test_pass(self):
        with mock.patch.object(cli, "_ollama_models", return_value=[classify.ROUTER_MODEL]):
            code, out = run(ENV, [".env"])
        self.assertEqual(code, 0, out)

    def test_bedrock_needs_aws(self):
        with mock.patch.object(cli, "_ollama_models", return_value=[classify.ROUTER_MODEL]):
            code, out = run({**ENV, "THRIFT_HARD": "bedrock/m"}, [".env"])
            self.assertEqual(code, 1)
            code, _ = run({**ENV, "THRIFT_HARD": "bedrock/m", "AWS_PROFILE": "p"}, [".env"])
            self.assertEqual(code, 0)


if __name__ == "__main__":
    unittest.main()
