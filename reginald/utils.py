import re

from error import ReginaldException


def c_sanitize(s: str) -> str:
    return re.sub(r"\s", "_", s)


def c_fitting_unsigned_type(bitwidth: int) -> str:
    possible_variable_sizes = [8, 16, 32, 64]
    possible_variable_sizes = [size for size in possible_variable_sizes if size >= bitwidth]
    if len(possible_variable_sizes) == 0:
        raise ReginaldException(f"No valid c type found for to store {bitwidth} bits!")

    size = min(possible_variable_sizes)
    return f"uint{size}_t"
