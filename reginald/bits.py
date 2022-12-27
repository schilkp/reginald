from math import ceil, log2
from typing import List

import more_itertools
from pydantic import NonNegativeInt, PositiveInt
from pydantic.dataclasses import dataclass

from reginald.error import ReginaldException


@dataclass
class BitRange:
    lsb_position: NonNegativeInt
    width: PositiveInt

    def get_bitmask(self) -> int:
        return ((2 ** self.width) - 1) << self.lsb_position

    def get_bitlist(self) -> List[int]:
        return [self.lsb_position + i for i in range(self.width)]

    def contains_bit(self, bit: int) -> bool:
        return bit >= self.lsb_position and bit < (self.lsb_position + self.width)

    def __str__(self) -> str:
        if self.width == 1:
            return str(self.lsb_position)
        else:
            return f"{self.lsb_position+self.width-1}-{self.lsb_position}"


@dataclass
class Bits:
    ranges: List[BitRange]

    @classmethod
    def zero(cls):
        return Bits(ranges=[])

    @classmethod
    def from_range(cls, lsb_position: NonNegativeInt, width: PositiveInt):
        return Bits(ranges=[BitRange(lsb_position=lsb_position, width=width)])

    @classmethod
    def from_bitlist(cls, bitlist: List[NonNegativeInt]):
        bitlist = sorted(bitlist)
        ranges = []
        for group in more_itertools.consecutive_groups(bitlist):
            bitlist_group = list(group)
            ranges.append(BitRange(lsb_position=min(bitlist_group),
                          width=(max(bitlist_group) - min(bitlist_group) + 1)))

        return Bits(ranges=ranges)

    def get_bitlist(self) -> List[int]:
        bits = []
        for range in self.ranges:
            bits.extend(range.get_bitlist())
        return bits

    def get_bitmask(self) -> int:
        mask = 0
        for range in self.ranges:
            mask |= range.get_bitmask()
        return mask

    def get_unpositioned_bits(self):
        return self.bitwise_rshift(self.lsb_position())

    def get_bitranges(self) -> List[BitRange]:
        return self.ranges

    def get_bitrange(self) -> BitRange:
        if len(self.ranges) == 1:
            return self.ranges[0]
        else:
            breakpoint()
            raise ReginaldException(
                f"Cannot specify bit range for a field that is non-continous! (Field mask: {hex(self.get_bitmask())})")

    def lsb_position(self) -> NonNegativeInt:
        l = [r.lsb_position for r in self.ranges]
        if len(l) == 0:
            raise ValueError("Cannot get LSB of 0")
        return min(l)

    def msb_position(self) -> NonNegativeInt:
        l = [r.lsb_position + r.width - 1 for r in self.ranges]
        if len(l) == 0:
            raise ValueError("Cannot get MSB of 0")
        return max(l)

    def bitwise_lshift(self, amt: NonNegativeInt):
        bitlist = [b+amt for b in self.get_bitlist()]
        return Bits.from_bitlist(bitlist)

    def bitwise_rshift(self, amt: NonNegativeInt):
        bitlist = [b-amt for b in self.get_bitlist() if b-amt >= 0]
        return Bits.from_bitlist(bitlist)

    def bitwise_not(self, maximum_width: PositiveInt):
        if self.msb_position() + 1 > maximum_width:
            breakpoint()
            raise ValueError("Inversion width too small for range")

        bitlist_is = self.get_bitlist()
        bitlist_inv = []
        for bit in range(maximum_width):
            if not bit in bitlist_is:
                bitlist_inv.append(bit)

        return Bits.from_bitlist(bitlist_inv)

    @classmethod
    def bitwise_or(cls, a, b):
        bitlist_or = a.get_bitlist()

        for bit in b.get_bitlist():
            if not bit in bitlist_or:
                bitlist_or.append(bit)

        return Bits.from_bitlist(bitlist_or)

    @classmethod
    def bitwise_and(cls, a, b):
        bitlist_and = []

        bitlist_a = a.get_bitlist()
        bitlist_b = b.get_bitlist()

        for bit in bitlist_a:
            if bit in bitlist_b:
                bitlist_and.append(bit)

        return Bits.from_bitlist(bitlist_and)


def fits_into_bitwidth(val: int, bitwidth: int) -> bool:
    if val == 0:
        return True
    if val < 0:
        return ceil(log2(val*-1)) <= (bitwidth-1)
    else:
        return ceil(log2(val+1)) <= bitwidth
