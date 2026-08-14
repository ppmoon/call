from app.util import log
from app.worker import Worker


def orchestrate(msg: str) -> str:
    log(msg)
    return Worker().process(msg)
