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

    def get_bitlist(self) -> List[NonNegativeInt]:
        return [self.lsb_position + i for i in range(self.width)]

    def contains_bit(self, bit: NonNegativeInt) -> bool:
        return bit >= self.lsb_position and bit < (self.lsb_position + self.width)

    def extract_this_field_from(self, val: NonNegativeInt) -> NonNegativeInt:
        return (self.get_bitmask() & val) >> self.lsb_position

    def __str__(self) -> str:
        if self.width == 1:
            return str(self.lsb_position)
        else:
            return f"{self.lsb_position+self.width-1}-{self.lsb_position}"


@dataclass
class Bits:
    bitlist: List[NonNegativeInt]

    @classmethod
    def zero(cls):
        return Bits(bitlist=[])

    @classmethod
    def from_position(cls, lsb_position: NonNegativeInt, width: PositiveInt):
        return Bits(bitlist=BitRange(lsb_position=lsb_position, width=width).get_bitlist())

    @classmethod
    def from_bitrange(cls, range: BitRange):
        return Bits(bitlist=range.get_bitlist())

    @classmethod
    def from_bitranges(cls, ranges: List[BitRange]):
        bitlist = []
        for range in ranges:
            for bit in range.get_bitlist():
                if not bit in bitlist:
                    bitlist.append(bit)

        return Bits.from_bitlist(bitlist)

    @classmethod
    def from_bitlist(cls, bitlist: List[NonNegativeInt]):
        return Bits(bitlist=bitlist)

    def get_bitlist(self) -> List[NonNegativeInt]:
        return self.bitlist

    def get_bitmask(self) -> int:
        mask = 0
        for bit in self.bitlist:
            mask |= (1 << bit)
        return mask

    def get_unpositioned_bits(self):
        return self.bitwise_rshift(self.lsb_position())

    def get_bitranges(self) -> List[BitRange]:
        self.bitlist = sorted(self.bitlist)
        ranges = []
        for group in more_itertools.consecutive_groups(self.bitlist):
            bitlist_group = list(group)
            ranges.append(BitRange(lsb_position=min(bitlist_group),
                          width=(max(bitlist_group) - min(bitlist_group) + 1)))
        return ranges

    def get_bitrange(self) -> BitRange:

        ranges = self.get_bitranges()
        if len(ranges) == 1:
            return ranges[0]
        else:
            breakpoint()
            raise ReginaldException(
                f"Cannot specify bit range for a field that is non-continous! (Field mask: {hex(self.get_bitmask())})")

    def lsb_position(self) -> NonNegativeInt:
        if len(self.bitlist) == 0:
            raise ValueError("Cannot get LSB of 0")
        return min(self.bitlist)

    def msb_position(self) -> NonNegativeInt:
        if len(self.bitlist) == 0:
            raise ValueError("Cannot get MSB of 0")
        return max(self.bitlist)

    def bitwise_lshift(self, amt: NonNegativeInt):
        bitlist = [b+amt for b in self.bitlist]
        return Bits.from_bitlist(bitlist)

    def bitwise_rshift(self, amt: NonNegativeInt):
        bitlist = [b-amt for b in self.bitlist if b-amt >= 0]
        return Bits.from_bitlist(bitlist)

    def bitwise_not(self, maximum_width: PositiveInt):
        if self.msb_position() + 1 > maximum_width:
            breakpoint()
            raise ValueError("Inversion width too small for range")

        bitlist_is = self.bitlist
        bitlist_inv = []
        for bit in range(maximum_width):
            if not bit in bitlist_is:
                bitlist_inv.append(bit)

        return Bits.from_bitlist(bitlist_inv)

    @classmethod
    def bitwise_or(cls, a, b):
        bitlist_or = a.bitlist

        for bit in b.bitlist:
            if not bit in bitlist_or:
                bitlist_or.append(bit)

        return Bits.from_bitlist(bitlist_or)

    @classmethod
    def bitwise_and(cls, a, b):
        bitlist_and = []

        bitlist_a = a.bitlist
        bitlist_b = b.bitlist

        for bit in bitlist_a:
            if bit in bitlist_b:
                bitlist_and.append(bit)

        return Bits.from_bitlist(bitlist_and)

    def extract_this_field_from(self, val: NonNegativeInt) -> NonNegativeInt:
        return (self.get_bitmask() & val) >> self.lsb_position()


def fits_into_bitwidth(val: int, bitwidth: int) -> bool:
    if val == 0:
        return True
    if val < 0:
        return ceil(log2(val*-1)) <= (bitwidth-1)
    else:
        return ceil(log2(val+1)) <= bitwidth
