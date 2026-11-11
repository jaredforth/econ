import contextlib
import io
import os
import shutil
import tempfile
import unittest
from unittest import mock

from thrift import cli


def init(*flags):
    with contextlib.redirect_stdout(io.StringIO()), mock.patch.dict(os.environ, {}, clear=False):
        cli.main(["init", *flags])


class InitTest(unittest.TestCase):
    def setUp(self):
        self.old = os.getcwd()
        self.d = tempfile.mkdtemp()
        shutil.copy(os.path.join(self.old, ".env.example"), self.d)
        os.chdir(self.d)
        p = mock.patch.object(cli, "doctor")
        p.start()
        self.addCleanup(p.stop)

    def tearDown(self):
        os.chdir(self.old)
        shutil.rmtree(self.d)

    def test_local_then_idempotent(self):
        init("--local")
        a = open(".env").read()
        self.assertIn("LITELLM_MASTER_KEY=sk-", a)
        self.assertNotIn("sk-change-me", a)
        self.assertIn("THRIFT_HARD=ollama_chat/", a)
        self.assertNotIn("bedrock/", a)
        init("--local")
        self.assertEqual(a, open(".env").read())

    def test_bedrock(self):
        init("--bedrock")
        a = open(".env").read()
        self.assertIn("THRIFT_SIMPLE=bedrock/", a)
        self.assertNotIn("ollama_chat/", a)
