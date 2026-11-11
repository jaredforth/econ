import asyncio
import os
import unittest

from thrift.classify import last_user_text, pick_tier


class LastUserText(unittest.TestCase):
    def test_string(self):
        m = [{"role": "user", "content": "a"}, {"role": "assistant", "content": "b"},
             {"role": "user", "content": "c"}]
        self.assertEqual(last_user_text(m), "c")

    def test_blocks_joined(self):
        m = [{"role": "user", "content": [{"type": "text", "text": "x"}, {"type": "image"},
                                          {"type": "text", "text": "y"}]}]
        self.assertEqual(last_user_text(m), "x y")

    def test_skips_tool_result_turn(self):
        m = [{"role": "user", "content": "earlier"},
             {"role": "user", "content": [{"type": "tool_result", "content": "r"}]}]
        self.assertEqual(last_user_text(m), "earlier")

    def test_empty(self):
        self.assertEqual(last_user_text([]), "")


class PickTier(unittest.TestCase):
    def test_down_returns_moderate(self):
        os.environ["OLLAMA_HOST"] = "http://127.0.0.1:9"
        self.assertEqual(asyncio.run(pick_tier("x")), "moderate")


if __name__ == "__main__":
    unittest.main()
