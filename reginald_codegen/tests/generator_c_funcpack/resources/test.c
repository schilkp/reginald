#include "Unity/unity.h"
#include "Unity/unity_internals.h"
#include "out.h"
#include <string.h>

void reverse_array(uint8_t *from, uint8_t *to, size_t len) {
  for (size_t i = 0; i < len; i++) {
    to[i] = from[len - i - 1];
  }
}

// ====== TESTS ================================================================

void test_basic_reg1(void) {

  // Packing:
  struct chip_reg1 reg = {
      .field0 = true,
      .field1 = 0xA,
  };

  uint8_t expected_packed_reg = (0x1 << 0)  // Field 0
                                | 0xA << 2; // Field 1

  uint8_t packed_reg_le = {0};

#ifdef TEST_SKIP_GENERIC_MACROS
  chip_reg1_pack_le(&reg, &packed_reg_le);
#else
  CHIP_PACK_LE(&reg, &packed_reg_le);
#endif

  TEST_ASSERT_EQUAL_HEX8_ARRAY(&expected_packed_reg, &packed_reg_le, 1);

  uint8_t packed_reg_be = {0};

#ifdef TEST_SKIP_GENERIC_MACROS
  chip_reg1_pack_be(&reg, &packed_reg_be);
#else
  CHIP_PACK_BE(&reg, &packed_reg_be);
#endif

  TEST_ASSERT_EQUAL_HEX8_ARRAY(&expected_packed_reg, &packed_reg_be, 1);

  // Unpacking:
  packed_reg_le |= 0x3 << 6; // unused bits
  reg = chip_reg1_unpack_le(&packed_reg_le);
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(true, reg.field0, "(field0)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0xA, reg.field1, "(field1)");

  packed_reg_be |= 0x3 << 6; // unused bits
  reg = chip_reg1_unpack_be(&packed_reg_le);
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(true, reg.field0, "(field0)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0xA, reg.field1, "(field1)");

  // Try unpacking:
  memset(&reg, 0, sizeof(struct chip_reg1));
#ifdef TEST_SKIP_GENERIC_MACROS
  TEST_ASSERT_EQUAL(0, chip_reg1_try_unpack_le(&packed_reg_le, &reg));
#else
  TEST_ASSERT_EQUAL(0, CHIP_TRY_UNPACK_LE(&packed_reg_le, &reg));
#endif

  TEST_ASSERT_EQUAL_UINT8_MESSAGE(true, reg.field0, "(field0)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0xA, reg.field1, "(field1)");

  memset(&reg, 0, sizeof(struct chip_reg1));
#ifdef TEST_SKIP_GENERIC_MACROS
  TEST_ASSERT_EQUAL(0, chip_reg1_try_unpack_le(&packed_reg_be, &reg));
#else
  TEST_ASSERT_EQUAL(0, CHIP_TRY_UNPACK_BE(&packed_reg_be, &reg));
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

  uint8_t expected_packed_reg_le[2] = {
      [0] = ((0x1 << 4)              // Always write
             | (CHIP_STAT_HOT << 6)  // Field 1
             | (CHIP_FIELD2_EN << 0) // Field 2
             | (true << 2)           // Field 3
             ),
      [1] = ((0xA) << 0), // Field 4
  };
  uint8_t expected_packed_reg_be[2] = {0};
  reverse_array(expected_packed_reg_le, expected_packed_reg_be, 2);

  uint8_t packed_reg_le[2] = {0};
  chip_reg2_pack_le(&reg, packed_reg_le);
  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg_le, packed_reg_le, 2);

  uint8_t packed_reg_be[2] = {0};
  chip_reg2_pack_be(&reg, packed_reg_be);
  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg_be, packed_reg_be, 2);

  // Try unpacking:
  // Change values of unused bits. correct_endianess
  // converts to little endian and back if big endian testing is enabled.

  packed_reg_le[0] |= 0x3 << 4;        // always-write
  packed_reg_le[1] |= 0x7 << (13 - 8); // unused bits
  reverse_array(packed_reg_le, packed_reg_be, 2);

  memset(&reg, 0, sizeof(struct chip_reg2));
  TEST_ASSERT_EQUAL(0, chip_reg2_try_unpack_le(packed_reg_le, &reg));
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(CHIP_STAT_HOT, reg.field1, "(field1)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(CHIP_FIELD2_EN, reg.field2, "(field2)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(true, reg.field3, "(field3)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0xA, reg.field4, "(field4)");

  memset(&reg, 0, sizeof(struct chip_reg2));
  TEST_ASSERT_EQUAL(0, chip_reg2_try_unpack_be(packed_reg_be, &reg));
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(CHIP_STAT_HOT, reg.field1, "(field1)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(CHIP_FIELD2_EN, reg.field2, "(field2)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(true, reg.field3, "(field3)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0xA, reg.field4, "(field4)");
}

void test_basic_reg3(void) {

  // Packing:
  struct chip_reg3 reg = {.field0 = 0xCBF, .field1 = 0x1F};

  uint8_t expected_packed_reg_le[8] = {
      [0] = (0xBF), // Field 0
      [1] = (0x0C), // Field 0
      [7] = (0x1F), // Field 1
  };
  uint8_t expected_packed_reg_be[8] = {0};
  reverse_array(expected_packed_reg_le, expected_packed_reg_be, 8);

  uint8_t packed_reg_le[8] = {0};
  chip_reg3_pack_le(&reg, packed_reg_le);
  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg_le, packed_reg_le, 8);

  uint8_t packed_reg_be[8] = {0};
  chip_reg3_pack_be(&reg, packed_reg_be);
  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg_be, packed_reg_be, 8);

  // Unpacking:

  // Change values of unused bits:
  packed_reg_le[2] = 0xFF;
  packed_reg_le[3] = 0xFF;
  packed_reg_le[4] = 0xFF;
  packed_reg_le[5] = 0xFF;

  packed_reg_be[2] = 0xFF;
  packed_reg_be[3] = 0xFF;
  packed_reg_be[4] = 0xFF;
  packed_reg_be[5] = 0xFF;

  reg = chip_reg3_unpack_le(packed_reg_le);
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0xCBF, reg.field0, "(field0)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0x1F, reg.field1, "(field1)");

  reg = chip_reg3_unpack_be(packed_reg_be);
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0xCBF, reg.field0, "(field0)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0x1F, reg.field1, "(field1)");

  // Try unpacking:
  memset(&reg, 0, sizeof(struct chip_reg3));
  TEST_ASSERT_EQUAL(0, chip_reg3_try_unpack_le(packed_reg_le, &reg));
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0xCBF, reg.field0, "(field0)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0x1F, reg.field1, "(field1)");

  memset(&reg, 0, sizeof(struct chip_reg3));
  TEST_ASSERT_EQUAL(0, chip_reg3_try_unpack_be(packed_reg_be, &reg));
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0xCBF, reg.field0, "(field0)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0x1F, reg.field1, "(field1)");
}

void test_enum_validation(void) {
  TEST_ASSERT_EQUAL(false, CHIP_IS_VALID_STAT(0x0));
  TEST_ASSERT_EQUAL(true, CHIP_IS_VALID_STAT(0x1));
  TEST_ASSERT_EQUAL(true, CHIP_IS_VALID_STAT(0x2));
  TEST_ASSERT_EQUAL(true, CHIP_IS_VALID_STAT(0x3));
  TEST_ASSERT_EQUAL(false, CHIP_IS_VALID_STAT(0x4));

  TEST_ASSERT_EQUAL(false, CHIP_IS_VALID_FIELD2(0x0));
  TEST_ASSERT_EQUAL(false, CHIP_IS_VALID_FIELD2(0x1));
  TEST_ASSERT_EQUAL(false, CHIP_IS_VALID_FIELD2(0x2));
  TEST_ASSERT_EQUAL(true, CHIP_IS_VALID_FIELD2(0x3));
  TEST_ASSERT_EQUAL(false, CHIP_IS_VALID_FIELD2(0x4));
}

void test_shared_layout_basic(void) {
  struct chip_basic_shared_layout reg = {
      .shared_field1 = 0x4,
      .shared_field2 = CHIP_SHARED_FIELD2_IS_ONE,
  };

  // Packing:
  uint8_t expected_packed_reg_le[2] = {
      [0] = (uint8_t)(0x4U << 1),                             // shared_field1
      [1] = (uint8_t)(CHIP_SHARED_FIELD2_IS_ONE << (10 - 8)), // shared_field2
  };
  uint8_t expected_packed_reg_be[2] = {0};
  reverse_array(expected_packed_reg_le, expected_packed_reg_be, 2);

  uint8_t packed_reg_le[2] = {0};
  chip_basic_shared_layout_pack_le(&reg, packed_reg_le);
  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg_le, packed_reg_le, 2);

  uint8_t packed_reg_be[2] = {0};
  chip_basic_shared_layout_pack_be(&reg, packed_reg_be);
  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg_be, packed_reg_be, 2);

  // Unpacking
  reg = chip_basic_shared_layout_unpack_le(packed_reg_le);
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0x4, reg.shared_field1, "(shared_field1)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(CHIP_SHARED_FIELD2_IS_ONE, reg.shared_field2,
                                  "(shared_field2)");

  reg = chip_basic_shared_layout_unpack_be(packed_reg_be);
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0x4, reg.shared_field1, "(shared_field1)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(CHIP_SHARED_FIELD2_IS_ONE, reg.shared_field2,
                                  "(shared_field2)");
}

void test_fixed_across_bytes(void) {
  struct chip_reg_fixed_across_bytes reg = {0};

  // Packing:
  uint8_t expected_packed_reg_le[2] = {
      [0] = (0x1 << 6),
      [1] = (0x2),
  };
  uint8_t expected_packed_reg_be[2] = {0};
  reverse_array(expected_packed_reg_le, expected_packed_reg_be, 2);

  uint8_t packed_reg_le[2] = {0};
  chip_reg_fixed_across_bytes_pack_le(&reg, packed_reg_le);
  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg_le, packed_reg_le, 2);

  uint8_t packed_reg_be[2] = {0};
  chip_reg_fixed_across_bytes_pack_be(&reg, packed_reg_be);
  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg_be, packed_reg_be, 2);
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

  uint8_t expected_packed_reg_le[2] = {
      [0] = (uint8_t)(0x1U << 0 |  // layout_field.f1
                      0xFFU << 2), // layout_field.f2.f22
      [1] = (uint8_t)(0x3),        // layout_field.f2.f22
  };

  uint8_t expected_packed_reg_be[2] = {0};
  reverse_array(expected_packed_reg_le, expected_packed_reg_be, 2);

  uint8_t packed_reg_le[2] = {0};
  chip_reg_layout_field_pack_le(&reg, packed_reg_le);
  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg_le, packed_reg_le, 2);

  uint8_t packed_reg_be[2] = {0};
  chip_reg_layout_field_pack_be(&reg, packed_reg_be);
  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg_be, packed_reg_be, 2);

  // Unpacking
  reg = chip_reg_layout_field_unpack_le(packed_reg_le);
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0x1, reg.layout_field.f1,
                                  "(layout_field.f1)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0xFF, reg.layout_field.f2.f22,
                                  "(layout_field.f2.f22)");

  reg = chip_reg_layout_field_unpack_be(packed_reg_be);
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0x1, reg.layout_field.f1,
                                  "(layout_field.f1)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(0xFF, reg.layout_field.f2.f22,
                                  "(layout_field.f2.f22)");
}

void test_nested_only_fixed(void) {
  struct chip_reg_nested_only_fixed reg = {0};

  // Packing:

  uint8_t expected_packed_reg[1] = {[0] = 0xAB};

  uint8_t packed_reg_le[1] = {0};
  chip_reg_nested_only_fixed_pack_le(&reg, packed_reg_le);

  uint8_t packed_reg_be[1] = {0};
  chip_reg_nested_only_fixed_pack_be(&reg, packed_reg_be);

  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg, packed_reg_le, 1);
  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg, packed_reg_be, 1);
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

  uint8_t packed_reg_le[1] = {0};
  chip_reg_split_field_pack_le(&reg, packed_reg_le);
  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg, packed_reg_le, 1);

  uint8_t packed_reg_be[1] = {0};
  chip_reg_split_field_pack_be(&reg, packed_reg_be);
  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg, packed_reg_be, 1);

  // Unpacking:
  struct chip_reg_split_field reg_unpacked_le =
      chip_reg_split_field_unpack_le(packed_reg_le);
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(
      0x1 | (0x1 << 4), reg_unpacked_le.split_field_1, "(split_field_1)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(
      0x2 | (0x2 << 4), reg_unpacked_le.split_field_2, "(split_field_2)");

  struct chip_reg_split_field reg_unpacked_be =
      chip_reg_split_field_unpack_be(packed_reg_be);
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(
      0x1 | (0x1 << 4), reg_unpacked_be.split_field_1, "(split_field_1)");
  TEST_ASSERT_EQUAL_UINT8_MESSAGE(
      0x2 | (0x2 << 4), reg_unpacked_be.split_field_2, "(split_field_2)");
}

void test_split_enum(void) {
  struct chip_reg_split_enum reg = {.split_enum = CHIP_SPLIT_ENUM_SE_3};

  // Packing:

  uint8_t expected_packed_reg[1] = {[0] = 0x5};

  uint8_t packed_reg_le[1] = {0};
  chip_reg_split_enum_pack_le(&reg, packed_reg_le);
  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg, packed_reg_le, 1);

  uint8_t packed_reg_be[1] = {0};
  chip_reg_split_enum_pack_be(&reg, packed_reg_be);
  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg, packed_reg_be, 1);

  // Unpacking:

  uint8_t packed_reg = 0x5;
  reg = chip_reg_split_enum_unpack_le(&packed_reg);
  TEST_ASSERT_EQUAL_UINT8(CHIP_SPLIT_ENUM_SE_3, reg.split_enum);
  reg = chip_reg_split_enum_unpack_be(&packed_reg);
  TEST_ASSERT_EQUAL_UINT8(CHIP_SPLIT_ENUM_SE_3, reg.split_enum);

  packed_reg = 0x7;
  reg = chip_reg_split_enum_unpack_le(&packed_reg);
  TEST_ASSERT_EQUAL_UINT8(CHIP_SPLIT_ENUM_SE_3, reg.split_enum);
  reg = chip_reg_split_enum_unpack_be(&packed_reg);
  TEST_ASSERT_EQUAL_UINT8(CHIP_SPLIT_ENUM_SE_3, reg.split_enum);

  packed_reg = 0x0;
  reg = chip_reg_split_enum_unpack_le(&packed_reg);
  TEST_ASSERT_EQUAL_UINT8(CHIP_SPLIT_ENUM_SE_0, reg.split_enum);
  reg = chip_reg_split_enum_unpack_be(&packed_reg);
  TEST_ASSERT_EQUAL_UINT8(CHIP_SPLIT_ENUM_SE_0, reg.split_enum);

  packed_reg = 0xA;
  reg = chip_reg_split_enum_unpack_le(&packed_reg);
  TEST_ASSERT_EQUAL_UINT8(CHIP_SPLIT_ENUM_SE_0, reg.split_enum);
  reg = chip_reg_split_enum_unpack_be(&packed_reg);
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

  uint8_t packed_reg_le[1] = {0};
  chip_reg_split_layout_pack_le(&reg, packed_reg_le);
  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg, packed_reg_le, 1);

  uint8_t packed_reg_be[1] = {0};
  chip_reg_split_layout_pack_be(&reg, packed_reg_be);
  TEST_ASSERT_EQUAL_HEX8_ARRAY(expected_packed_reg, packed_reg_be, 1);

  // Unpacking:
  reg = chip_reg_split_layout_unpack_le(packed_reg_le);
  TEST_ASSERT_EQUAL_UINT8(0x3, reg.split_layout.f1);
  TEST_ASSERT_EQUAL_UINT8(0x7, reg.split_layout.f2);

  reg = chip_reg_split_layout_unpack_be(packed_reg_be);
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
  RUN_TEST(test_enum_validation);
  RUN_TEST(test_shared_layout_basic);
  RUN_TEST(test_fixed_across_bytes);
  RUN_TEST(test_layout_fields);
  RUN_TEST(test_nested_only_fixed);
  RUN_TEST(test_split_field);
  RUN_TEST(test_split_enum);
  RUN_TEST(test_split_layout);
  return UNITY_END();
}
