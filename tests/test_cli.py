import io
import unittest
from contextlib import redirect_stdout
from unittest import mock

from thrift import cli


async def _hard(text):
    _hard.seen = text
    return "hard"


class TestCli(unittest.TestCase):
    def run_cli(self, argv):
        buf = io.StringIO()
        with mock.patch.object(cli, "pick_tier", _hard), redirect_stdout(buf):
            cli.main(argv)
        return buf.getvalue().splitlines()

    def test_route_arg(self):
        with mock.patch.dict("os.environ", {"BEDROCK_HARD": "m-hard"}):
            self.assertEqual(self.run_cli(["route", "hi"]), ["hard", "m-hard"])

    def test_route_no_model(self):
        with mock.patch.dict("os.environ", {}, clear=True):
            self.assertEqual(self.run_cli(["route", "hi"]), ["hard"])

    def test_route_stdin(self):
        with mock.patch("sys.stdin", io.StringIO("from stdin")):
            self.assertEqual(self.run_cli(["route", "-"])[0], "hard")
        self.assertEqual(_hard.seen, "from stdin")


if __name__ == "__main__":
    unittest.main()
