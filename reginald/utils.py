import re


def c_sanitize(s: str) -> str:
    return re.sub(r"\s", "_", s)
