#include "Unity/unity.h"
#include "Unity/unity_internals.h"
#include "output/generated.h"

#include <string.h>

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

  uint8_t packed_reg = {0};

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
  memset(&reg, 0, sizeof(struct chip_reg1));
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

  uint8_t packed_reg[2] = {0};

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

  // Try unpacking:
  // Change values of unused bits. correct_endianess
  // converts to little endian and back if big endian testing is enabled.
  correct_endianess(packed_reg, 2);
  packed_reg[0] |= 0x3 << 4;        // always-write
  packed_reg[1] |= 0x7 << (13 - 8); // unused bits
  correct_endianess(packed_reg, 2);

  memset(&reg, 0, sizeof(struct chip_reg2));
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

  uint8_t packed_reg[8] = {0};

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
  memset(&reg, 0, sizeof(struct chip_reg3));
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

void test_shared_layout_basic(void) {
  struct chip_basic_shared_layout reg = {
      .shared_field1 = 0x4,
      .shared_field2 = CHIP_SHARED_FIELD2_IS_ONE,
  };

  // Packing:
  uint8_t expected_packed_reg[2] = {
      [0] = (uint8_t)(0x4U << 1),                             // shared_field1
      [1] = (uint8_t)(CHIP_SHARED_FIELD2_IS_ONE << (10 - 8)), // shared_field2
  };
  correct_endianess(expected_packed_reg, 2);

  uint8_t packed_reg[2] = {0};
  chip_basic_shared_layout_pack(&reg, packed_reg);
  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg, packed_reg, 2);

  // Unpacking
  reg = chip_basic_shared_layout_unpack(packed_reg);
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0x4, reg.shared_field1, "(shared_field1)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(CHIP_SHARED_FIELD2_IS_ONE, reg.shared_field2,
                                  "(shared_field2)");
}

void test_fixed_across_bytes(void) {
  struct chip_reg_fixed_across_bytes reg = {0};

  // Packing:
  uint8_t expected_packed_reg[2] = {
      [0] = (0x1 << 6),
      [1] = (0x2),
  };
  correct_endianess(expected_packed_reg, 2);

  uint8_t packed_reg[2] = {0};
  chip_reg_fixed_across_bytes_pack(&reg, packed_reg);
  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg, packed_reg, 2);
}

void test_layout_fields(void) {
  struct chip_reg_layout_field reg = {
      .layout_field =
          {
              .f1 = 1,
              .f2 = {.f22 = 0xFF},
          },
  };

  // Packing:
  uint8_t packed_reg[2] = {0};

  chip_reg_layout_field_pack(&reg, packed_reg);

  uint8_t expected_packed_reg[2] = {
      [0] = (uint8_t)(0x1U << 0 |  // layout_field.f1
                      0xFFU << 2), // layout_field.f2.f22
      [1] = (uint8_t)(0x3),        // layout_field.f2.f22
  };
  correct_endianess(expected_packed_reg, 2);

  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg, packed_reg, 2);

  // Unpacking
  reg = chip_reg_layout_field_unpack(packed_reg);
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0x1, reg.layout_field.f1,
                                  "(layout_field.f1)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0xFF, reg.layout_field.f2.f22,
                                  "(layout_field.f2.f22)");
}

void test_nested_only_fixed(void) {
  struct chip_reg_nested_only_fixed reg = {0};

  // Packing:
  uint8_t packed_reg[1] = {0};
  chip_reg_nested_only_fixed_pack(&reg, packed_reg);

  uint8_t expected_packed_reg[1] = {[0] = 0xAB};
  correct_endianess(expected_packed_reg, 1);

  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg, packed_reg, 1);
}

void test_split_field(void) {
  struct chip_reg_split_field reg = {
      .split_field_1 = (0x1 << 0) | (0x1 << 4),
      .split_field_2 = (0x2 << 0) | (0x2 << 4),
  };

  // Packing:

  uint8_t expected_packed_reg[1] = {[0] = (0x1 << 0) | // Field 1
                                          (0x1 << 4) | // Field 1
                                          (0x2 << 2) | // Field 2
                                          (0x2 << 6)}; // Field 2

  uint8_t packed_reg[1] = {0};
  chip_reg_split_field_pack(&reg, packed_reg);
  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg, packed_reg, 1);

  // Unpacking:
  struct chip_reg_split_field reg_unpacked =
      chip_reg_split_field_unpack(packed_reg);
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0x1 | (0x1 << 4), reg_unpacked.split_field_1,
                                  "(split_field_1)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0x2 | (0x2 << 4), reg_unpacked.split_field_2,
                                  "(split_field_2)");
}

void test_split_enum(void) {
  struct chip_reg_split_enum reg = {.split_enum = CHIP_SPLIT_ENUM_SE_3};

  // Packing:

  uint8_t expected_packed_reg[1] = {[0] = 0x5};

  uint8_t packed_reg[1] = {0};
  chip_reg_split_enum_pack(&reg, packed_reg);
  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg, packed_reg, 1);

  // Unpacking:

  packed_reg[0] = 0x5;
  reg = chip_reg_split_enum_unpack(packed_reg);
  TEST_ASSERT_EQUAL_UINT8(CHIP_SPLIT_ENUM_SE_3, reg.split_enum);

  packed_reg[0] = 0x7;
  reg = chip_reg_split_enum_unpack(packed_reg);
  TEST_ASSERT_EQUAL_UINT8(CHIP_SPLIT_ENUM_SE_3, reg.split_enum);

  packed_reg[0] = 0x0;
  reg = chip_reg_split_enum_unpack(packed_reg);
  TEST_ASSERT_EQUAL_UINT8(CHIP_SPLIT_ENUM_SE_0, reg.split_enum);

  packed_reg[0] = 0xA;
  reg = chip_reg_split_enum_unpack(packed_reg);
  TEST_ASSERT_EQUAL_UINT8(CHIP_SPLIT_ENUM_SE_0, reg.split_enum);
}

void test_split_layout(void) {
  struct chip_reg_split_layout reg = {
      .split_layout =
          {
              .f1 = 3,
              .f2 = 7,
          },
  };

  // Packing:
  uint8_t expected_packed_reg[1] = {[0] = (0x3 << 2) | (0x7 << 5)};

  uint8_t packed_reg[1] = {0};
  chip_reg_split_layout_pack(&reg, packed_reg);
  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg, packed_reg, 1);

  // Unpacking:
  reg = chip_reg_split_layout_unpack(packed_reg);
  TEST_ASSERT_EQUAL_UINT8(0x3, reg.split_layout.f1);
  TEST_ASSERT_EQUAL_UINT8(0x7, reg.split_layout.f2);
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
  RUN_TEST(test_shared_layout_basic);
  RUN_TEST(test_fixed_across_bytes);
  RUN_TEST(test_layout_fields);
  RUN_TEST(test_nested_only_fixed);
  RUN_TEST(test_split_field);
  RUN_TEST(test_split_enum);
  RUN_TEST(test_split_layout);
  return UNITY_END();
}
