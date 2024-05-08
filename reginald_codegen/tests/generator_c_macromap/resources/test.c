#include "Unity/unity.h"
#include "Unity/unity_internals.h"
#include "out.h"

// ====== TESTS ================================================================

void test_basic_reg1(void) {
  uint16_t expected = (0x1 << 0) | (0xA << 2);

  uint16_t is = 0;
  is &= ~CHIP_REG1_FIELD0_MASK;
  is |= (0x1 << CHIP_REG1_FIELD0_SHIFT) & CHIP_REG1_FIELD0_MASK;
  is &= ~CHIP_REG1_FIELD1_MASK;
  is |= (0xA << CHIP_REG1_FIELD1_SHIFT) & CHIP_REG1_FIELD1_MASK;

  TEST_ASSERT_EQUAL_HEX16(expected, is);
}

void test_basic_reg2(void) {
  uint16_t expected = (0x1 << 4)      // Always write
                      | (0x3 << 6)    // Field 1 (= STAT_HOT)
                      | (0x3 << 0)    // Field 2 (= EN)
                      | (1 << 2)      // Field 3
                      | ((0xA) << 8); // Field 4

  uint16_t is = 0;
  is &= ~CHIP_REG2_FIELD1_MASK;
  is |= (CHIP_REG2_FIELD1_VAL_HOT << CHIP_REG2_FIELD1_SHIFT) &
        CHIP_REG2_FIELD1_MASK;

  is &= ~CHIP_REG2_FIELD2_MASK;
  is |= (CHIP_REG2_FIELD2_VAL_EN << CHIP_REG2_FIELD2_SHIFT) &
        CHIP_REG2_FIELD2_MASK;

  is &= ~CHIP_REG2_FIELD3_MASK;
  is |= (0x1 << CHIP_REG2_FIELD3_SHIFT) & CHIP_REG2_FIELD3_MASK;

  is &= ~CHIP_REG2_FIELD4_MASK;
  is |= (0xA << CHIP_REG2_FIELD4_SHIFT) & CHIP_REG2_FIELD4_MASK;

  is &= ~CHIP_REG2_ALWAYSWRITE_MASK;
  is |= CHIP_REG2_ALWAYSWRITE_VALUE;

  TEST_ASSERT_EQUAL_HEX16(expected, is);
}

// ======= MAIN ================================================================

void setUp(void) {}

void tearDown(void) {}

int main(void) {
  UNITY_BEGIN();
  RUN_TEST(test_basic_reg1);
  RUN_TEST(test_basic_reg2);
  return UNITY_END();
}
