#include "Unity/unity.h"
#include "Unity/unity_internals.h"
#include "output/generated.h"

#ifdef TEST_BIG_ENDIAN
#include <string.h>
#endif

void correct_endianess(uint8_t *arr, size_t len) {
#ifdef TEST_BIG_ENDIAN
  uint8_t buf[len];
  for (size_t i = 0; i < len; i++) {
    buf[i] = arr[len - i - 1];
  }
  memcpy(arr, buf, len);
#else
  (void)arr;
  (void)len;
#endif
}

// ====== TESTS ================================================================

void test_basic_reg1(void) {

  // Packing:
  struct chip_reg1 reg = {
      .field0 = true,
      .field1 = 0xA,
  };

  uint8_t packed_reg;

#ifdef TEST_SKIP_GENERIC_MACROS
  chip_reg1_pack(&reg, &packed_reg);
#else
  CHIP_PACK(&reg, &packed_reg);
#endif

  uint8_t expected_packed_reg = (0x1 << 0)  // Field 0
                                | 0xA << 2; // Field 1

  TEST_ASSERT_EQUAL_HEX8_ARRAY(&expected_packed_reg, &packed_reg, 1);

  // Unpacking:
  // Change values of unused bits.
  packed_reg |= 0x3 << 5; // unused bits

  reg = chip_reg1_unpack(&packed_reg);
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(true, reg.field0, "(field0)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0xA, reg.field1, "(field1)");

  // Try unpacking:
#ifdef TEST_SKIP_GENERIC_MACROS
  TEST_ASSERT_EQUAL(chip_reg1_try_unpack(&packed_reg, &reg), 0);
#else
  TEST_ASSERT_EQUAL(CHIP_TRY_UNPACK(&packed_reg, &reg), 0);
#endif

  TEST_ASSERT_EQUAL_UINT8_MESSAGE(true, reg.field0, "(field0)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0xA, reg.field1, "(field1)");
}

void test_basic_reg2(void) {
  // Packing:
  struct chip_reg2 reg = {
      .field1 = CHIP_STAT_HOT,
      .field2 = CHIP_FIELD2_EN,
      .field3 = true,
      .field4 = 0xA,
  };

  uint8_t packed_reg[2];

#ifdef TEST_SKIP_GENERIC_MACROS
  chip_reg2_pack(&reg, packed_reg);
#else
  CHIP_PACK(&reg, packed_reg);
#endif

  uint8_t expected_packed_reg[2] = {
      [0] = ((0x1 << 4)              // Always write
             | (CHIP_STAT_HOT << 6)  // Field 1
             | (CHIP_FIELD2_EN << 0) // Field 2
             | (true << 2)           // Field 3
             ),
      [1] = ((0xA) << 0), // Field 4
  };
  correct_endianess(expected_packed_reg, 2);

  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg, packed_reg, 2);

  // Unpacking:
  // Change values of unused bits. correct_endianess
  // converts to little endian and back if big endian testing is enabled.
  correct_endianess(packed_reg, 2);
  packed_reg[0] |= 0x3 << 4;        // always-write
  packed_reg[1] |= 0x7 << (13 - 8); // unused bits
  correct_endianess(packed_reg, 2);

  reg = chip_reg2_unpack(packed_reg);
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(CHIP_STAT_HOT, reg.field1, "(field1)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(CHIP_FIELD2_EN, reg.field2, "(field2)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(true, reg.field3, "(field3)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0xA, reg.field4, "(field4)");

  // Try unpacking:
#ifdef TEST_SKIP_GENERIC_MACROS
  TEST_ASSERT_EQUAL(chip_reg2_try_unpack(packed_reg, &reg), 0);
#else
  TEST_ASSERT_EQUAL(CHIP_TRY_UNPACK(packed_reg, &reg), 0);
#endif

  TEST_ASSERT_EQUAL_UINT8_MESSAGE(CHIP_STAT_HOT, reg.field1, "(field1)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(CHIP_FIELD2_EN, reg.field2, "(field2)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(true, reg.field3, "(field3)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0xA, reg.field4, "(field4)");
}

void test_basic_reg3(void) {

  // Packing:
  struct chip_reg3 reg = {.field0 = 0xCBF, .field1 = 0x1F};

  uint8_t packed_reg[8];

#ifdef TEST_SKIP_GENERIC_MACROS
  chip_reg3_pack(&reg, packed_reg);
#else
  CHIP_PACK(&reg, packed_reg);
#endif

  uint8_t expected_packed_reg[8] = {
      [0] = (0xBF), // Field 0
      [1] = (0x0C), // Field 0
      [7] = (0x1F), // Field 1
  };
  correct_endianess(expected_packed_reg, 8);

  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg, packed_reg, 8);

  // Unpacking:
  // Change values of unused bits. correct_endianess
  // converts to little endian and back if big endian testing is enabled.
  correct_endianess(packed_reg, 8);
  packed_reg[2] = 0xFF;
  packed_reg[3] = 0xFF;
  packed_reg[4] = 0xFF;
  packed_reg[5] = 0xFF;
  correct_endianess(packed_reg, 8);

  reg = chip_reg3_unpack(packed_reg);
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0xCBF, reg.field0, "(field0)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0x1F, reg.field1, "(field1)");

  // Try unpacking:
#ifdef TEST_SKIP_GENERIC_MACROS
  TEST_ASSERT_EQUAL(chip_reg3_try_unpack(packed_reg, &reg), 0);
#else
  TEST_ASSERT_EQUAL(CHIP_TRY_UNPACK(packed_reg, &reg), 0);
#endif

  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0xCBF, reg.field0, "(field0)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0x1F, reg.field1, "(field1)");
}

void test_stat_enum_validation(void) {
  TEST_ASSERT_EQUAL(chip_is_stat_enum(0x0), false);
  TEST_ASSERT_EQUAL(chip_is_stat_enum(0x1), true);
  TEST_ASSERT_EQUAL(chip_is_stat_enum(0x2), true);
  TEST_ASSERT_EQUAL(chip_is_stat_enum(0x3), true);
  TEST_ASSERT_EQUAL(chip_is_stat_enum(0x4), false);
}

void test_field2_enum_validation(void) {
  TEST_ASSERT_EQUAL(chip_is_field2_enum(0x0), false);
  TEST_ASSERT_EQUAL(chip_is_field2_enum(0x1), false);
  TEST_ASSERT_EQUAL(chip_is_field2_enum(0x2), false);
  TEST_ASSERT_EQUAL(chip_is_field2_enum(0x3), true);
  TEST_ASSERT_EQUAL(chip_is_field2_enum(0x4), false);
}

// ======= MAIN ================================================================

void setUp(void) {}

void tearDown(void) {}

int main(void) {
  UNITY_BEGIN();
  RUN_TEST(test_basic_reg1);
  RUN_TEST(test_basic_reg2);
  RUN_TEST(test_basic_reg3);
  RUN_TEST(test_stat_enum_validation);
  RUN_TEST(test_field2_enum_validation);
  return UNITY_END();
}
