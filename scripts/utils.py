import pathlib


def get_repo_dir():
    path = pathlib.Path(__file__).resolve()
    while path != pathlib.Path(path.root):
        path = path.parent
        if path.joinpath("Cargo.toml").exists():
            return path

    raise RuntimeError("Could not find Cargo.toml in any parent dir")
