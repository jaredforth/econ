import os
import sys
import tempfile
import unittest
from unittest import mock

from thrift import cli


class ServeTest(unittest.TestCase):
    def run_in(self, files, argv=("serve",)):
        with tempfile.TemporaryDirectory() as d:
            for f in files:
                open(os.path.join(d, f), "w").close()
            old = os.getcwd()
            os.chdir(d)
            try:
                with mock.patch("os.execvp") as ex, mock.patch.dict(os.environ, {"LITELLM_MASTER_KEY": "k"}):
                    os.environ.pop("THRIFT_HOME", None)
                    cli.main(list(argv))
                    return ex
            finally:
                os.chdir(old)

    def test_argv(self):
        ex = self.run_in(["config.yaml"], ["serve", "--port", "5000"])
        ex.assert_called_once_with(
            sys.executable,
            [sys.executable, "-m", "litellm.proxy.proxy_cli", "--config", "config.yaml", "--host", "127.0.0.1", "--port", "5000"],
        )

    def test_missing_config_exits_2(self):
        with self.assertRaises(SystemExit) as c:
            self.run_in([])
        self.assertEqual(c.exception.code, 2)


if __name__ == "__main__":
    unittest.main()
