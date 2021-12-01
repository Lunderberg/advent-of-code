#!/usr/bin/env python3

import argparse
import datetime
import requests
import os
import time

from bs4 import BeautifulSoup

from utils import get_repo_dir


class AOCDownloader:
    def __init__(
        self, year=2021, session_id=None, throttle_period=datetime.timedelta(seconds=5)
    ):
        self.year = year
        self.session_id = session_id or os.environ["AOC_SESSION_ID"]
        self.throttle_period = throttle_period
        self.wait_until = datetime.datetime.now()

    def download_file(self, url):
        repo_dir = get_repo_dir()
        cache_file = repo_dir.joinpath(".cache", self.session_id, url.replace("/", "_"))
        if cache_file.exists():
            return cache_file.open().read()

        self.wait_for_throttle()
        res = requests.get(url, cookies={"session": self.session_id})
        self.reset_throttle()

        res.raise_for_status()

        cache_file.parent.mkdir(parents=True, exist_ok=True)
        with cache_file.open("w") as f:
            f.write(res.text)

        return res.text

    def wait_for_throttle(self):
        now = datetime.datetime.now()
        if now < self.wait_until:
            time.sleep((self.wait_until - now).total_seconds())

    def reset_throttle(self):
        self.wait_until = datetime.datetime.now() + self.throttle_period

    def problem_released(self, day):
        est = datetime.timezone(datetime.timedelta(hours=-5))
        problem_release = datetime.datetime(self.year, 12, day, tzinfo=est)
        now = datetime.datetime.now(tz=est)
        return now > problem_release

    def download_input(self, day):
        if not self.problem_released(day):
            return
        url = f"https://adventofcode.com/{self.year}/day/{day}/input"
        return self.download_file(url)

    def download_examples(self, day):
        if not self.problem_released(day):
            return

        url = f"https://adventofcode.com/{self.year}/day/{day}"
        html = self.download_file(url)
        soup = BeautifulSoup(html, features="lxml")

        examples = []
        for code_block in soup.find_all("pre"):
            previous_paragraph = code_block.previous_sibling
            while True:
                if (
                    hasattr(previous_paragraph, "get_text")
                    and previous_paragraph.get_text().strip()
                ):
                    break
                previous_paragraph = previous_paragraph.previous_sibling

            if "example" in previous_paragraph.get_text().lower():
                examples.append(code_block.get_text())

        return examples


def main(args):
    downloader = AOCDownloader(year=2020)
    repo_dir = get_repo_dir()

    for day in range(1, 26):
        if list(repo_dir.joinpath("inputs").glob(f"day{day:02d}_example*.txt")):
            continue

        examples = downloader.download_examples(day)

        for i, example in enumerate(examples):
            out_file = repo_dir.joinpath("inputs", f"day{day:02d}_example{i+1}.txt")
            out_file.parent.mkdir(parents=True, exist_ok=True)
            with out_file.open("w") as f:
                f.write(example)
            print(f"Saved example {i} of day {day:02d}")

    for day in range(1, 26):
        output_file = repo_dir.joinpath("inputs", f"day{day:02d}.txt")
        if output_file.exists():
            continue

        text = downloader.download_input(day)
        if text:
            output_file.parent.mkdir(parents=True, exist_ok=True)
            with open(output_file, "w") as f:
                f.write(text)
            print(f"Saved day {day} to {output_file}")


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
