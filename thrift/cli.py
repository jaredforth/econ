import argparse
import asyncio
import os
import sys

from thrift.classify import pick_tier


def route(args):
    text = sys.stdin.read() if args.prompt == "-" else args.prompt
    tier = asyncio.run(pick_tier(text))
    print(tier)
    model = os.environ.get(f"THRIFT_{tier.upper()}")
    if model:
        print(model)


def main(argv=None):
    p = argparse.ArgumentParser(prog="thrift")
    sub = p.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("route", help="print the tier (and model) for a prompt")
    r.add_argument("prompt", help="prompt text, or - to read stdin")
    r.set_defaults(fn=route)
    args = p.parse_args(argv)
    args.fn(args)
    return 0


if __name__ == "__main__":
    main()
