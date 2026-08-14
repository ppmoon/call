from httpcli.fetch import fetch


def main(url: str = "https://example.com") -> str:
    return fetch(url)
