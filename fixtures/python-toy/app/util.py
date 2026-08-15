import os


def log(msg: str) -> None:
    print(f"{msg} cwd={os.getcwd()}")
