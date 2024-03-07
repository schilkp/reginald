#include "generated.h"
#include <assert.h>

int main(void) {

  // ====== REG1 ===============================================================

  // Basic packing:
  struct chip_reg1 reg1 = {
      .field1 = CHIP_STAT_HOT,
      .field2 = CHIP_FIELD2_EN,
      .field3 = true,
      .field4 = 0xA,
  };

#ifdef GENERIC_REGINALD_FUNCS
  uint16_t packed_reg1 = CHIP_PACK(&reg1);
#else
  uint16_t packed_reg1 = chip_reg1_pack(&reg1);
#endif

  uint16_t expected_reg1 = (0x1 << 4)    // Always write
                           | (0x3 << 6)  // Field 1 == "HOT"
                           | (0x3 << 0)  // Field 2 == "EN"
                           | (0x1 << 2)  // Field 2 == "EN"
                           | (0xA << 8); // Field 4 == 0xA
  assert(packed_reg1 == expected_reg1);

  // Basic unpacking:
  packed_reg1 |= 0x7 << 13; // futz with unused bits
  packed_reg1 |= 0x3 << 4;  // futz with always-write

#ifdef GENERIC_REGINALD_FUNCS
  CHIP_UNPACK_INTO(packed_reg1, &reg1);
#else
  chip_reg1_unpack_into(packed_reg1, &reg1);
#endif
  assert(reg1.field1 == CHIP_STAT_HOT);
  assert(reg1.field2 == CHIP_FIELD2_EN);
  assert(reg1.field3 == true);
  assert(reg1.field4 == 0xA);

  struct chip_reg1 reg1_copy = CHIP_REG1_UNPACK((uint64_t)packed_reg1);
  assert(reg1_copy.field1 == CHIP_STAT_HOT);
  assert(reg1_copy.field2 == CHIP_FIELD2_EN);
  assert(reg1_copy.field3 == true);
  assert(reg1_copy.field4 == 0xA);

  // Enum validation:
  assert(chip_can_unpack_enum_stat(0x0) == false);
  assert(chip_can_unpack_enum_stat(0x1) == true);
  assert(chip_can_unpack_enum_stat(0x2) == true);
  assert(chip_can_unpack_enum_stat(0x3) == true);
  assert(chip_can_unpack_enum_stat(0x4) == false);
  assert(chip_can_unpack_enum_field2(0x0) == false);
  assert(chip_can_unpack_enum_field2(0x3) == true);

  // ====== REG2 ===============================================================

  // Basic packing:
  struct chip_reg2 reg2 = {
      .field0 = true,
  };

#ifdef GENERIC_REGINALD_FUNCS
  uint8_t packed_reg2 = CHIP_PACK(&reg2);
#else
  uint8_t packed_reg2 = chip_reg2_pack(&reg2);
#endif

  uint16_t expected_reg2 = (0x1 << 0); // Field 0
  assert(packed_reg2 == expected_reg2);

  // Basic unpacking:
  packed_reg2 |= 0xF << 1; // futz with unused bits

#ifdef GENERIC_REGINALD_FUNCS
  CHIP_UNPACK_INTO(packed_reg2, &reg2);
#else
  chip_reg2_unpack_into(packed_reg2, &reg2);
#endif

  assert(reg2.field0 == true);

  struct chip_reg2 reg2_copy = CHIP_REG2_UNPACK((uint64_t)packed_reg2);
  assert(reg2_copy.field0 == true);

  // ====== REG3 ===============================================================

  // Basic packing:
  struct chip_reg3 reg3 = {
      .field0 = 10,
  };
#ifdef GENERIC_REGINALD_FUNCS
  uint8_t packed_reg3 = CHIP_PACK(&reg3);
#else
  uint8_t packed_reg3 = chip_reg3_pack(&reg3);
#endif

  uint8_t expected_reg3 = (10 << 0); // Field 0
  assert(packed_reg3 == expected_reg3);

  // Basic unpacking:
  packed_reg3 |= 0xF << 4; // futz with unused bits

#ifdef GENERIC_REGINALD_FUNCS
  CHIP_UNPACK_INTO(packed_reg3, &reg3);
#else
  chip_reg3_unpack_into(packed_reg3, &reg3);
#endif
  assert(reg3.field0 == 10);

  struct chip_reg3 reg3_copy = CHIP_REG3_UNPACK((uint64_t)packed_reg3);
  assert(reg3_copy.field0 == 10);

  // ====== REG4 ===============================================================
  // Empty register, should not generate structs/funcs.

  // ====== BIGREG =============================================================

  // Basic packing:
  struct chip_bigreg bigreg = {.field0 = 0xFFF, .field1 = 0x1F};

#ifdef GENERIC_REGINALD_FUNCS
  uint64_t packed_bigreg = CHIP_PACK(&bigreg);
#else
  uint64_t packed_bigreg = chip_bigreg_pack(&bigreg);
#endif

  uint64_t expected_bigreg = 0x1F00000000000FFF;
  assert(packed_bigreg == expected_bigreg);

  // Basic unpacking:
  packed_bigreg |= 0xF << 20; // futz with unused bits

#ifdef GENERIC_REGINALD_FUNCS
  CHIP_UNPACK_INTO(packed_bigreg, &bigreg);
#else
  chip_bigreg_unpack_into(packed_bigreg, &bigreg);
#endif

  assert(bigreg.field0 == 0xFFF);
  assert(bigreg.field1 == 0x1F);

  struct chip_bigreg bigreg_copy = CHIP_BIGREG_UNPACK((uint64_t)packed_bigreg);
  assert(bigreg_copy.field0 == 0xFFF);
  assert(bigreg_copy.field1 == 0x1F);

  return 0;
}
