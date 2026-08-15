import json
import os


def fetch(url: str) -> str:
    """Return a fake HTTP body. Wow Script target: add retry here."""
    if not url:
        raise ValueError("url is required")
    payload = {"url": url, "cwd": os.getcwd()}
    return json.dumps(payload)
