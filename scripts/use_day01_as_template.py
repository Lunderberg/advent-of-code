#!/usr/bin/env python3

import argparse
import re


from utils import get_repo_dir


def main(args):
    repo_dir = get_repo_dir()
    solutions_dir = repo_dir.joinpath("src", "solutions")
    template = solutions_dir.joinpath("day01.rs").open().read()
    for day in range(2, 26):
        text = template.replace("Day01", f"Day{day:02d}").replace("  1", f"  {day}")
        text = re.sub(
            "".join(
                r"(?P<before>fn day\(&self\) -> i32 \{\s*)" r"1" r"(?P<after>\s*\})",
            ),
            f"\g<before>{day}\g<after>",
            text,
        )
        text = re.sub(
            "".join(
                r"(?P<before>fn implemented\(&self\) -> bool \{\s*)"
                r"true"
                r"(?P<after>\s*\})",
            ),
            f"\g<before>false\g<after>",
            text,
        )
        filepath = solutions_dir.joinpath(f"day{day:02d}.rs")
        with open(filepath, "w") as f:
            f.write(text)


def arg_main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--pdb",
        action="store_true",
        help="Start a pdb post mortem on uncaught exception",
    )

    args = parser.parse_args()

    try:
        main(args)
    except Exception:
        if args.pdb:
            import pdb, traceback

            traceback.print_exc()
            pdb.post_mortem()
        raise


if __name__ == "__main__":
    arg_main()
