export const example_code = ` // clang-format off
/**
 * @file out.c
 * @brief chip
 * @note do not edit directly: generated using reginald from ./reginald_codegen/tests/map.yaml.
 *
 * Generator: c.funcpack
 */
#ifndef REGINALD_OUT_C
#define REGINALD_OUT_C

#include <stdint.h>
#include <stdbool.h>

//===----------------------------------------------------------------------===//
// Shared Enums
//===----------------------------------------------------------------------===//

// ---- STAT Enum --------------------------------------------------------------

enum chip_stat {
  CHIP_STAT_COOL = 0x1U,
  CHIP_STAT_HOT = 0x3U,
  CHIP_STAT_NOT_COOL = 0x2U,
};

/**
 * @brief Check if a numeric value is a valid @ref enum chip_stat.
 * @returns bool (true/false)
 */
#define CHIP_IS_VALID_STAT(_VAL_) (                                                                \\
  (0x1U <= (_VAL_) && (_VAL_) <= 0x3U) ? true :                                                    \\
  false )

//===----------------------------------------------------------------------===//
// Shared Layout Structs
//===----------------------------------------------------------------------===//

// ---- BASIC_SHARED_LAYOUT Layout ---------------------------------------------
// Fields:
//  - [4:1] SHARED_FIELD1 (uint)
//  - [10] SHARED_FIELD2 (enum SHARED_FIELD2)

// Layout-specific enums and sub-layouts:

enum chip_shared_field2 {
  CHIP_SHARED_FIELD2_IS_ONE = 0x1U,
  CHIP_SHARED_FIELD2_IS_ZERO = 0x0U,
};

// Layout Struct:

/** @note use pack/unpack functions for conversion to/from packed binary value */
struct chip_basic_shared_layout {
  uint8_t shared_field1;
  enum chip_shared_field2 shared_field2;
};

// Enum validation functions:

/**
 * @brief Check if a numeric value is a valid @ref enum chip_shared_field2.
 * @returns bool (true/false)
 */
#define CHIP_IS_VALID_SHARED_FIELD2(_VAL_) (                                                       \\
  ((_VAL_) <= 0x1U) ? true :                                                                       \\
  false )

// Layout struct conversion functions:

/**
 * @brief Convert @ref struct chip_basic_shared_layout struct to packed little-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_basic_shared_layout_pack_le(const struct chip_basic_shared_layout *r, uint8_t val[2]) {
  // SHARED_FIELD1 @ basic_shared_layout[4:1]:
  val[0] &= (uint8_t)~0x1EU;
  val[0] |= (uint8_t)(((uint8_t)(r->shared_field1 << 1)) & 0x1EU);
  // SHARED_FIELD2 @ basic_shared_layout[10]:
  val[1] &= (uint8_t)~0x4U;
  val[1] |= (uint8_t)(((uint8_t)(r->shared_field2 << 2)) & 0x4U);
}

/**
 * @brief Convert @ref struct chip_basic_shared_layout struct to packed big-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_basic_shared_layout_pack_be(const struct chip_basic_shared_layout *r, uint8_t val[2]) {
  // SHARED_FIELD1 @ basic_shared_layout[4:1]:
  val[1] &= (uint8_t)~0x1EU;
  val[1] |= (uint8_t)(((uint8_t)(r->shared_field1 << 1)) & 0x1EU);
  // SHARED_FIELD2 @ basic_shared_layout[10]:
  val[0] &= (uint8_t)~0x4U;
  val[0] |= (uint8_t)(((uint8_t)(r->shared_field2 << 2)) & 0x4U);
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_basic_shared_layout chip_basic_shared_layout_unpack_le(const uint8_t val[2]) {
  struct chip_basic_shared_layout r = {0};
  // SHARED_FIELD1 @ basic_shared_layout[4:1]:
  r.shared_field1 = (uint8_t)(((val[0] & 0x1EU) >> 1));
  // SHARED_FIELD2 @ basic_shared_layout[10]:
  r.shared_field2 = (enum chip_shared_field2)(((val[1] & 0x4U) >> 2));
  return r;
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_basic_shared_layout chip_basic_shared_layout_unpack_be(const uint8_t val[2]) {
  struct chip_basic_shared_layout r = {0};
  // SHARED_FIELD1 @ basic_shared_layout[4:1]:
  r.shared_field1 = (uint8_t)(((val[1] & 0x1EU) >> 1));
  // SHARED_FIELD2 @ basic_shared_layout[10]:
  r.shared_field2 = (enum chip_shared_field2)(((val[0] & 0x4U) >> 2));
  return r;
}

/**
 * @brief Validate struct
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 * Confirms that all enums are valid, and all values fit into respective fields
 */
static inline int chip_validate_basic_shared_layout(const struct chip_basic_shared_layout *r) {
  if ((r->shared_field1 & ~(uint8_t)0xF) != 0) return 2;
  if (!(CHIP_IS_VALID_SHARED_FIELD2(r->shared_field2))) return 11;
  return 0;
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_basic_shared_layout_try_unpack_le(const uint8_t val[2], struct chip_basic_shared_layout *r) {
  *r = chip_basic_shared_layout_unpack_le(val);
  return chip_validate_basic_shared_layout(r);
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_basic_shared_layout_try_unpack_be(const uint8_t val[2], struct chip_basic_shared_layout *r) {
  *r = chip_basic_shared_layout_unpack_be(val);
  return chip_validate_basic_shared_layout(r);
}

//===----------------------------------------------------------------------===//
// REG1 Register
//===----------------------------------------------------------------------===//
// Fields:
//  - [0] FIELD0 (bool)
//  - [5:2] FIELD1 (uint)

#define CHIP_REG1_ADDRESS  (0x0U) //!< REG1 register address
#define CHIP_REG1_RESET_LE {0x0U} //!< REG1 register reset value
#define CHIP_REG1_RESET_BE {0x0U} //!< REG1 register reset value

// Register Layout Struct:

/** @note use pack/unpack functions for conversion to/from packed binary value */
struct chip_reg1 {
  bool field0;
  uint8_t field1;
};

// Layout struct conversion functions:

/**
 * @brief Convert @ref struct chip_reg1 struct to packed little-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_reg1_pack_le(const struct chip_reg1 *r, uint8_t val[1]) {
  // FIELD0 @ reg1[0]:
  val[0] &= (uint8_t)~0x1U;
  val[0] |= (uint8_t)(((uint8_t)r->field0) & 0x1U);
  // FIELD1 @ reg1[5:2]:
  val[0] &= (uint8_t)~0x3CU;
  val[0] |= (uint8_t)(((uint8_t)(r->field1 << 2)) & 0x3CU);
}

/**
 * @brief Convert @ref struct chip_reg1 struct to packed big-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_reg1_pack_be(const struct chip_reg1 *r, uint8_t val[1]) {
  // FIELD0 @ reg1[0]:
  val[0] &= (uint8_t)~0x1U;
  val[0] |= (uint8_t)(((uint8_t)r->field0) & 0x1U);
  // FIELD1 @ reg1[5:2]:
  val[0] &= (uint8_t)~0x3CU;
  val[0] |= (uint8_t)(((uint8_t)(r->field1 << 2)) & 0x3CU);
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_reg1 chip_reg1_unpack_le(const uint8_t val[1]) {
  struct chip_reg1 r = {0};
  // FIELD0 @ reg1[0]:
  r.field0 = (bool)((val[0] & 0x1U));
  // FIELD1 @ reg1[5:2]:
  r.field1 = (uint8_t)(((val[0] & 0x3CU) >> 2));
  return r;
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_reg1 chip_reg1_unpack_be(const uint8_t val[1]) {
  struct chip_reg1 r = {0};
  // FIELD0 @ reg1[0]:
  r.field0 = (bool)((val[0] & 0x1U));
  // FIELD1 @ reg1[5:2]:
  r.field1 = (uint8_t)(((val[0] & 0x3CU) >> 2));
  return r;
}

/**
 * @brief Validate struct
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 * Confirms that all enums are valid, and all values fit into respective fields
 */
static inline int chip_validate_reg1(const struct chip_reg1 *r) {
  if ((r->field1 & ~(uint8_t)0xF) != 0) return 3;
  return 0;
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_reg1_try_unpack_le(const uint8_t val[1], struct chip_reg1 *r) {
  *r = chip_reg1_unpack_le(val);
  return chip_validate_reg1(r);
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_reg1_try_unpack_be(const uint8_t val[1], struct chip_reg1 *r) {
  *r = chip_reg1_unpack_be(val);
  return chip_validate_reg1(r);
}

//===----------------------------------------------------------------------===//
// REG2 Register
//===----------------------------------------------------------------------===//
// Fields:
//  - [1:0] FIELD2 (enum FIELD2)
//  - [2] FIELD3 (bool)
//  - [5:4] RESERVED (fixed: 0x1)
//  - [7:6] FIELD1 (enum STAT)
//  - [12:8] FIELD4 (uint)

#define CHIP_REG2_ADDRESS  (0x0U)        //!< REG2 register address
#define CHIP_REG2_RESET_LE {0x43U, 0x0U} //!< REG2 register reset value
#define CHIP_REG2_RESET_BE {0x0U, 0x43U} //!< REG2 register reset value

// Register-specific enums and sub-layouts:

enum chip_field2 {
  CHIP_FIELD2_EN = 0x3U,
};

// Register Layout Struct:

/** @note use pack/unpack functions for conversion to/from packed binary value */
struct chip_reg2 {
  enum chip_stat field1;
  enum chip_field2 field2;
  bool field3;
  uint8_t field4;
};

// Enum validation functions:

/**
 * @brief Check if a numeric value is a valid @ref enum chip_field2.
 * @returns bool (true/false)
 */
#define CHIP_IS_VALID_FIELD2(_VAL_) (                                                              \\
  ((_VAL_) == 0x3U) ? true :                                                                       \\
  false )

// Layout struct conversion functions:

/**
 * @brief Convert @ref struct chip_reg2 struct to packed little-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_reg2_pack_le(const struct chip_reg2 *r, uint8_t val[2]) {
  // FIELD1 @ reg2[7:6]:
  val[0] &= (uint8_t)~0xC0U;
  val[0] |= (uint8_t)(((uint8_t)(r->field1 << 6)) & 0xC0U);
  // FIELD2 @ reg2[1:0]:
  val[0] &= (uint8_t)~0x3U;
  val[0] |= (uint8_t)(((uint8_t)r->field2) & 0x3U);
  // FIELD3 @ reg2[2]:
  val[0] &= (uint8_t)~0x4U;
  val[0] |= (uint8_t)(((uint8_t)(r->field3 << 2)) & 0x4U);
  // FIELD4 @ reg2[12:8]:
  val[1] &= (uint8_t)~0x1FU;
  val[1] |= (uint8_t)(((uint8_t)r->field4) & 0x1FU);
  // RESERVED @ reg2[5:4]:
  val[0] &= (uint8_t)~0x30U;
  val[0] |= (uint8_t)0x10; // Fixed value.
}

/**
 * @brief Convert @ref struct chip_reg2 struct to packed big-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_reg2_pack_be(const struct chip_reg2 *r, uint8_t val[2]) {
  // FIELD1 @ reg2[7:6]:
  val[1] &= (uint8_t)~0xC0U;
  val[1] |= (uint8_t)(((uint8_t)(r->field1 << 6)) & 0xC0U);
  // FIELD2 @ reg2[1:0]:
  val[1] &= (uint8_t)~0x3U;
  val[1] |= (uint8_t)(((uint8_t)r->field2) & 0x3U);
  // FIELD3 @ reg2[2]:
  val[1] &= (uint8_t)~0x4U;
  val[1] |= (uint8_t)(((uint8_t)(r->field3 << 2)) & 0x4U);
  // FIELD4 @ reg2[12:8]:
  val[0] &= (uint8_t)~0x1FU;
  val[0] |= (uint8_t)(((uint8_t)r->field4) & 0x1FU);
  // RESERVED @ reg2[5:4]:
  val[1] &= (uint8_t)~0x30U;
  val[1] |= (uint8_t)0x10; // Fixed value.
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_reg2 chip_reg2_unpack_le(const uint8_t val[2]) {
  struct chip_reg2 r = {0};
  // FIELD1 @ reg2[7:6]:
  r.field1 = (enum chip_stat)(((val[0] & 0xC0U) >> 6));
  // FIELD2 @ reg2[1:0]:
  r.field2 = (enum chip_field2)((val[0] & 0x3U));
  // FIELD3 @ reg2[2]:
  r.field3 = (bool)(((val[0] & 0x4U) >> 2));
  // FIELD4 @ reg2[12:8]:
  r.field4 = (uint8_t)((val[1] & 0x1FU));
  return r;
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_reg2 chip_reg2_unpack_be(const uint8_t val[2]) {
  struct chip_reg2 r = {0};
  // FIELD1 @ reg2[7:6]:
  r.field1 = (enum chip_stat)(((val[1] & 0xC0U) >> 6));
  // FIELD2 @ reg2[1:0]:
  r.field2 = (enum chip_field2)((val[1] & 0x3U));
  // FIELD3 @ reg2[2]:
  r.field3 = (bool)(((val[1] & 0x4U) >> 2));
  // FIELD4 @ reg2[12:8]:
  r.field4 = (uint8_t)((val[0] & 0x1FU));
  return r;
}

/**
 * @brief Validate struct
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 * Confirms that all enums are valid, and all values fit into respective fields
 */
static inline int chip_validate_reg2(const struct chip_reg2 *r) {
  if (!(CHIP_IS_VALID_STAT(r->field1))) return 7;
  if (!(CHIP_IS_VALID_FIELD2(r->field2))) return 1;
  if ((r->field4 & ~(uint8_t)0x1F) != 0) return 9;
  return 0;
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_reg2_try_unpack_le(const uint8_t val[2], struct chip_reg2 *r) {
  *r = chip_reg2_unpack_le(val);
  return chip_validate_reg2(r);
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_reg2_try_unpack_be(const uint8_t val[2], struct chip_reg2 *r) {
  *r = chip_reg2_unpack_be(val);
  return chip_validate_reg2(r);
}

//===----------------------------------------------------------------------===//
// REG3 Register
//===----------------------------------------------------------------------===//
// Fields:
//  - [15:0] FIELD0 (uint)
//  - [63:56] FIELD1 (uint)

#define CHIP_REG3_ADDRESS (0x10U) //!< REG3 register address

// Register Layout Struct:

/** @note use pack/unpack functions for conversion to/from packed binary value */
struct chip_reg3 {
  uint16_t field0;
  uint8_t field1;
};

// Layout struct conversion functions:

/**
 * @brief Convert @ref struct chip_reg3 struct to packed little-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_reg3_pack_le(const struct chip_reg3 *r, uint8_t val[8]) {
  // FIELD0 @ reg3[15:0]:
  val[0] &= (uint8_t)~0xFFU;
  val[0] |= (uint8_t)(((uint8_t)r->field0) & 0xFFU);
  val[1] &= (uint8_t)~0xFFU;
  val[1] |= (uint8_t)(((uint8_t)(r->field0 >> 8)) & 0xFFU);
  // FIELD1 @ reg3[63:56]:
  val[7] &= (uint8_t)~0xFFU;
  val[7] |= (uint8_t)(((uint8_t)r->field1) & 0xFFU);
}

/**
 * @brief Convert @ref struct chip_reg3 struct to packed big-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_reg3_pack_be(const struct chip_reg3 *r, uint8_t val[8]) {
  // FIELD0 @ reg3[15:0]:
  val[6] &= (uint8_t)~0xFFU;
  val[6] |= (uint8_t)(((uint8_t)(r->field0 >> 8)) & 0xFFU);
  val[7] &= (uint8_t)~0xFFU;
  val[7] |= (uint8_t)(((uint8_t)r->field0) & 0xFFU);
  // FIELD1 @ reg3[63:56]:
  val[0] &= (uint8_t)~0xFFU;
  val[0] |= (uint8_t)(((uint8_t)r->field1) & 0xFFU);
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_reg3 chip_reg3_unpack_le(const uint8_t val[8]) {
  struct chip_reg3 r = {0};
  // FIELD0 @ reg3[15:0]:
  r.field0 = (uint16_t)(((uint16_t)(val[0] & 0xFFU)) | (((uint16_t)(val[1] & 0xFFU)) << 8));
  // FIELD1 @ reg3[63:56]:
  r.field1 = (uint8_t)((val[7] & 0xFFU));
  return r;
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_reg3 chip_reg3_unpack_be(const uint8_t val[8]) {
  struct chip_reg3 r = {0};
  // FIELD0 @ reg3[15:0]:
  r.field0 = (uint16_t)((((uint16_t)(val[6] & 0xFFU)) << 8) | ((uint16_t)(val[7] & 0xFFU)));
  // FIELD1 @ reg3[63:56]:
  r.field1 = (uint8_t)((val[0] & 0xFFU));
  return r;
}

/**
 * @brief Validate struct
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 * Confirms that all enums are valid, and all values fit into respective fields
 */
static inline int chip_validate_reg3(const struct chip_reg3 *r) {
  if ((r->field0 & ~(uint16_t)0xFFFF) != 0) return 1;
  if ((r->field1 & ~(uint8_t)0xFF) != 0) return 57;
  return 0;
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_reg3_try_unpack_le(const uint8_t val[8], struct chip_reg3 *r) {
  *r = chip_reg3_unpack_le(val);
  return chip_validate_reg3(r);
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_reg3_try_unpack_be(const uint8_t val[8], struct chip_reg3 *r) {
  *r = chip_reg3_unpack_be(val);
  return chip_validate_reg3(r);
}

//===----------------------------------------------------------------------===//
// REG_EMPTY Register
//===----------------------------------------------------------------------===//
// Fields:


#define CHIP_REG_EMPTY_ADDRESS  (0x10U) //!< REG_EMPTY register address
#define CHIP_REG_EMPTY_RESET_LE {0x0U}  //!< REG_EMPTY register reset value
#define CHIP_REG_EMPTY_RESET_BE {0x0U}  //!< REG_EMPTY register reset value

// Register Layout Struct:

/** @note use pack/unpack functions for conversion to/from packed binary value */
struct chip_reg_empty {
  int dummy; // Register contains no variable fields.
};

// Layout struct conversion functions:

/**
 * @brief Convert @ref struct chip_reg_empty struct to packed little-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_reg_empty_pack_le(const struct chip_reg_empty *r, uint8_t val[1]) {
  (void)val;
  (void)r;
}

/**
 * @brief Convert @ref struct chip_reg_empty struct to packed big-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_reg_empty_pack_be(const struct chip_reg_empty *r, uint8_t val[1]) {
  (void)val;
  (void)r;
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_reg_empty chip_reg_empty_unpack_le(const uint8_t val[1]) {
  struct chip_reg_empty r = {0};
  (void)val;
  (void)r;
  return r;
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_reg_empty chip_reg_empty_unpack_be(const uint8_t val[1]) {
  struct chip_reg_empty r = {0};
  (void)val;
  (void)r;
  return r;
}

/**
 * @brief Validate struct
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 * Confirms that all enums are valid, and all values fit into respective fields
 */
static inline int chip_validate_reg_empty(const struct chip_reg_empty *r) {
  (void)r;
  return 0;
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_reg_empty_try_unpack_le(const uint8_t val[1], struct chip_reg_empty *r) {
  *r = chip_reg_empty_unpack_le(val);
  return chip_validate_reg_empty(r);
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_reg_empty_try_unpack_be(const uint8_t val[1], struct chip_reg_empty *r) {
  *r = chip_reg_empty_unpack_be(val);
  return chip_validate_reg_empty(r);
}

//===----------------------------------------------------------------------===//
// REG_FIELD_BIGGER_THAN_ENUM_1 Register
//===----------------------------------------------------------------------===//
// Fields:
//  - [3:0] TINY_ENUM_1 (enum TINY_ENUM_1)

#define CHIP_REG_FIELD_BIGGER_THAN_ENUM_1_ADDRESS (0x21U) //!< REG_FIELD_BIGGER_THAN_ENUM_1 register address

// Register-specific enums and sub-layouts:

enum chip_tiny_enum_1 {
  CHIP_TINY_ENUM_1_F0 = 0x0U,
  CHIP_TINY_ENUM_1_F1 = 0x1U,
};

// Register Layout Struct:

/** @note use pack/unpack functions for conversion to/from packed binary value */
struct chip_reg_field_bigger_than_enum_1 {
  enum chip_tiny_enum_1 tiny_enum_1;
};

// Enum validation functions:

/**
 * @brief Check if a numeric value is a valid @ref enum chip_tiny_enum_1.
 * @returns bool (true/false)
 */
#define CHIP_IS_VALID_TINY_ENUM_1(_VAL_) (                                                         \\
  ((_VAL_) <= 0x1U) ? true :                                                                       \\
  false )

// Layout struct conversion functions:

/**
 * @brief Convert @ref struct chip_reg_field_bigger_than_enum_1 struct to packed little-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_reg_field_bigger_than_enum_1_pack_le(const struct chip_reg_field_bigger_than_enum_1 *r, uint8_t val[4]) {
  // TINY_ENUM_1 @ reg_field_bigger_than_enum_1[3:0]:
  val[0] &= (uint8_t)~0xFU;
  val[0] |= (uint8_t)(((uint8_t)r->tiny_enum_1) & 0xFU);
}

/**
 * @brief Convert @ref struct chip_reg_field_bigger_than_enum_1 struct to packed big-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_reg_field_bigger_than_enum_1_pack_be(const struct chip_reg_field_bigger_than_enum_1 *r, uint8_t val[4]) {
  // TINY_ENUM_1 @ reg_field_bigger_than_enum_1[3:0]:
  val[3] &= (uint8_t)~0xFU;
  val[3] |= (uint8_t)(((uint8_t)r->tiny_enum_1) & 0xFU);
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_reg_field_bigger_than_enum_1 chip_reg_field_bigger_than_enum_1_unpack_le(const uint8_t val[4]) {
  struct chip_reg_field_bigger_than_enum_1 r = {0};
  // TINY_ENUM_1 @ reg_field_bigger_than_enum_1[3:0]:
  r.tiny_enum_1 = (enum chip_tiny_enum_1)((val[0] & 0xFU));
  return r;
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_reg_field_bigger_than_enum_1 chip_reg_field_bigger_than_enum_1_unpack_be(const uint8_t val[4]) {
  struct chip_reg_field_bigger_than_enum_1 r = {0};
  // TINY_ENUM_1 @ reg_field_bigger_than_enum_1[3:0]:
  r.tiny_enum_1 = (enum chip_tiny_enum_1)((val[3] & 0xFU));
  return r;
}

/**
 * @brief Validate struct
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 * Confirms that all enums are valid, and all values fit into respective fields
 */
static inline int chip_validate_reg_field_bigger_than_enum_1(const struct chip_reg_field_bigger_than_enum_1 *r) {
  if (!(CHIP_IS_VALID_TINY_ENUM_1(r->tiny_enum_1))) return 1;
  return 0;
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_reg_field_bigger_than_enum_1_try_unpack_le(const uint8_t val[4], struct chip_reg_field_bigger_than_enum_1 *r) {
  *r = chip_reg_field_bigger_than_enum_1_unpack_le(val);
  return chip_validate_reg_field_bigger_than_enum_1(r);
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_reg_field_bigger_than_enum_1_try_unpack_be(const uint8_t val[4], struct chip_reg_field_bigger_than_enum_1 *r) {
  *r = chip_reg_field_bigger_than_enum_1_unpack_be(val);
  return chip_validate_reg_field_bigger_than_enum_1(r);
}

//===----------------------------------------------------------------------===//
// REG_FIELD_BIGGER_THAN_ENUM_2 Register
//===----------------------------------------------------------------------===//
// Fields:
//  - [31:0] TINY_ENUM_2 (enum TINY_ENUM_2)

#define CHIP_REG_FIELD_BIGGER_THAN_ENUM_2_ADDRESS (0x21U) //!< REG_FIELD_BIGGER_THAN_ENUM_2 register address

// Register-specific enums and sub-layouts:

/** @name TINY_ENUM_2 */
///@{
#define CHIP_TINY_ENUM_2_F0 (0x0U)
#define CHIP_TINY_ENUM_2_F1 (0x1U)
///@}

// Register Layout Struct:

/** @note use pack/unpack functions for conversion to/from packed binary value */
struct chip_reg_field_bigger_than_enum_2 {
  uint32_t tiny_enum_2;
};

// Enum validation functions:

/**
 * @brief Check if a numeric value is a valid @ref TINY_ENUM_2.
 * @returns bool (true/false)
 */
#define CHIP_IS_VALID_TINY_ENUM_2(_VAL_) (                                                         \\
  ((_VAL_) <= 0x1U) ? true :                                                                       \\
  false )

// Layout struct conversion functions:

/**
 * @brief Convert @ref struct chip_reg_field_bigger_than_enum_2 struct to packed little-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_reg_field_bigger_than_enum_2_pack_le(const struct chip_reg_field_bigger_than_enum_2 *r, uint8_t val[4]) {
  // TINY_ENUM_2 @ reg_field_bigger_than_enum_2[31:0]:
  val[0] &= (uint8_t)~0xFFU;
  val[0] |= (uint8_t)(((uint8_t)r->tiny_enum_2) & 0xFFU);
  val[1] &= (uint8_t)~0xFFU;
  val[1] |= (uint8_t)(((uint8_t)(r->tiny_enum_2 >> 8)) & 0xFFU);
  val[2] &= (uint8_t)~0xFFU;
  val[2] |= (uint8_t)(((uint8_t)(r->tiny_enum_2 >> 16)) & 0xFFU);
  val[3] &= (uint8_t)~0xFFU;
  val[3] |= (uint8_t)(((uint8_t)(r->tiny_enum_2 >> 24)) & 0xFFU);
}

/**
 * @brief Convert @ref struct chip_reg_field_bigger_than_enum_2 struct to packed big-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_reg_field_bigger_than_enum_2_pack_be(const struct chip_reg_field_bigger_than_enum_2 *r, uint8_t val[4]) {
  // TINY_ENUM_2 @ reg_field_bigger_than_enum_2[31:0]:
  val[0] &= (uint8_t)~0xFFU;
  val[0] |= (uint8_t)(((uint8_t)(r->tiny_enum_2 >> 24)) & 0xFFU);
  val[1] &= (uint8_t)~0xFFU;
  val[1] |= (uint8_t)(((uint8_t)(r->tiny_enum_2 >> 16)) & 0xFFU);
  val[2] &= (uint8_t)~0xFFU;
  val[2] |= (uint8_t)(((uint8_t)(r->tiny_enum_2 >> 8)) & 0xFFU);
  val[3] &= (uint8_t)~0xFFU;
  val[3] |= (uint8_t)(((uint8_t)r->tiny_enum_2) & 0xFFU);
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_reg_field_bigger_than_enum_2 chip_reg_field_bigger_than_enum_2_unpack_le(const uint8_t val[4]) {
  struct chip_reg_field_bigger_than_enum_2 r = {0};
  // TINY_ENUM_2 @ reg_field_bigger_than_enum_2[31:0]:
  r.tiny_enum_2 = ((uint32_t)(val[0] & 0xFFU)) | (((uint32_t)(val[1] & 0xFFU)) << 8) | (((uint32_t)(val[2] & 0xFFU)) << 16) | (((uint32_t)(val[3] & 0xFFU)) << 24);
  return r;
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_reg_field_bigger_than_enum_2 chip_reg_field_bigger_than_enum_2_unpack_be(const uint8_t val[4]) {
  struct chip_reg_field_bigger_than_enum_2 r = {0};
  // TINY_ENUM_2 @ reg_field_bigger_than_enum_2[31:0]:
  r.tiny_enum_2 = (((uint32_t)(val[0] & 0xFFU)) << 24) | (((uint32_t)(val[1] & 0xFFU)) << 16) | (((uint32_t)(val[2] & 0xFFU)) << 8) | ((uint32_t)(val[3] & 0xFFU));
  return r;
}

/**
 * @brief Validate struct
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 * Confirms that all enums are valid, and all values fit into respective fields
 */
static inline int chip_validate_reg_field_bigger_than_enum_2(const struct chip_reg_field_bigger_than_enum_2 *r) {
  if (!(CHIP_IS_VALID_TINY_ENUM_2(r->tiny_enum_2))) return 1;
  return 0;
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_reg_field_bigger_than_enum_2_try_unpack_le(const uint8_t val[4], struct chip_reg_field_bigger_than_enum_2 *r) {
  *r = chip_reg_field_bigger_than_enum_2_unpack_le(val);
  return chip_validate_reg_field_bigger_than_enum_2(r);
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_reg_field_bigger_than_enum_2_try_unpack_be(const uint8_t val[4], struct chip_reg_field_bigger_than_enum_2 *r) {
  *r = chip_reg_field_bigger_than_enum_2_unpack_be(val);
  return chip_validate_reg_field_bigger_than_enum_2(r);
}

//===----------------------------------------------------------------------===//
// REG_FIELD_BIGGER_THAN_ENUM_3 Register
//===----------------------------------------------------------------------===//
// Fields:
//  - [63:0] TINY_ENUM_3 (enum TINY_ENUM_3)

#define CHIP_REG_FIELD_BIGGER_THAN_ENUM_3_ADDRESS (0x21U) //!< REG_FIELD_BIGGER_THAN_ENUM_3 register address

// Register-specific enums and sub-layouts:

/** @name TINY_ENUM_3 */
///@{
#define CHIP_TINY_ENUM_3_F0 (0x0U)
#define CHIP_TINY_ENUM_3_F1 (0x1U)
///@}

// Register Layout Struct:

/** @note use pack/unpack functions for conversion to/from packed binary value */
struct chip_reg_field_bigger_than_enum_3 {
  uint64_t tiny_enum_3;
};

// Enum validation functions:

/**
 * @brief Check if a numeric value is a valid @ref TINY_ENUM_3.
 * @returns bool (true/false)
 */
#define CHIP_IS_VALID_TINY_ENUM_3(_VAL_) (                                                         \\
  ((_VAL_) <= 0x1U) ? true :                                                                       \\
  false )

// Layout struct conversion functions:

/**
 * @brief Convert @ref struct chip_reg_field_bigger_than_enum_3 struct to packed little-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_reg_field_bigger_than_enum_3_pack_le(const struct chip_reg_field_bigger_than_enum_3 *r, uint8_t val[8]) {
  // TINY_ENUM_3 @ reg_field_bigger_than_enum_3[63:0]:
  val[0] &= (uint8_t)~0xFFU;
  val[0] |= (uint8_t)(((uint8_t)r->tiny_enum_3) & 0xFFU);
  val[1] &= (uint8_t)~0xFFU;
  val[1] |= (uint8_t)(((uint8_t)(r->tiny_enum_3 >> 8)) & 0xFFU);
  val[2] &= (uint8_t)~0xFFU;
  val[2] |= (uint8_t)(((uint8_t)(r->tiny_enum_3 >> 16)) & 0xFFU);
  val[3] &= (uint8_t)~0xFFU;
  val[3] |= (uint8_t)(((uint8_t)(r->tiny_enum_3 >> 24)) & 0xFFU);
  val[4] &= (uint8_t)~0xFFU;
  val[4] |= (uint8_t)(((uint8_t)(r->tiny_enum_3 >> 32)) & 0xFFU);
  val[5] &= (uint8_t)~0xFFU;
  val[5] |= (uint8_t)(((uint8_t)(r->tiny_enum_3 >> 40)) & 0xFFU);
  val[6] &= (uint8_t)~0xFFU;
  val[6] |= (uint8_t)(((uint8_t)(r->tiny_enum_3 >> 48)) & 0xFFU);
  val[7] &= (uint8_t)~0xFFU;
  val[7] |= (uint8_t)(((uint8_t)(r->tiny_enum_3 >> 56)) & 0xFFU);
}

/**
 * @brief Convert @ref struct chip_reg_field_bigger_than_enum_3 struct to packed big-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_reg_field_bigger_than_enum_3_pack_be(const struct chip_reg_field_bigger_than_enum_3 *r, uint8_t val[8]) {
  // TINY_ENUM_3 @ reg_field_bigger_than_enum_3[63:0]:
  val[0] &= (uint8_t)~0xFFU;
  val[0] |= (uint8_t)(((uint8_t)(r->tiny_enum_3 >> 56)) & 0xFFU);
  val[1] &= (uint8_t)~0xFFU;
  val[1] |= (uint8_t)(((uint8_t)(r->tiny_enum_3 >> 48)) & 0xFFU);
  val[2] &= (uint8_t)~0xFFU;
  val[2] |= (uint8_t)(((uint8_t)(r->tiny_enum_3 >> 40)) & 0xFFU);
  val[3] &= (uint8_t)~0xFFU;
  val[3] |= (uint8_t)(((uint8_t)(r->tiny_enum_3 >> 32)) & 0xFFU);
  val[4] &= (uint8_t)~0xFFU;
  val[4] |= (uint8_t)(((uint8_t)(r->tiny_enum_3 >> 24)) & 0xFFU);
  val[5] &= (uint8_t)~0xFFU;
  val[5] |= (uint8_t)(((uint8_t)(r->tiny_enum_3 >> 16)) & 0xFFU);
  val[6] &= (uint8_t)~0xFFU;
  val[6] |= (uint8_t)(((uint8_t)(r->tiny_enum_3 >> 8)) & 0xFFU);
  val[7] &= (uint8_t)~0xFFU;
  val[7] |= (uint8_t)(((uint8_t)r->tiny_enum_3) & 0xFFU);
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_reg_field_bigger_than_enum_3 chip_reg_field_bigger_than_enum_3_unpack_le(const uint8_t val[8]) {
  struct chip_reg_field_bigger_than_enum_3 r = {0};
  // TINY_ENUM_3 @ reg_field_bigger_than_enum_3[63:0]:
  r.tiny_enum_3 = ((uint64_t)(val[0] & 0xFFU)) | (((uint64_t)(val[1] & 0xFFU)) << 8) | (((uint64_t)(val[2] & 0xFFU)) << 16) | (((uint64_t)(val[3] & 0xFFU)) << 24) | (((uint64_t)(val[4] & 0xFFU)) << 32) | (((uint64_t)(val[5] & 0xFFU)) << 40) | (((uint64_t)(val[6] & 0xFFU)) << 48) | (((uint64_t)(val[7] & 0xFFU)) << 56);
  return r;
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_reg_field_bigger_than_enum_3 chip_reg_field_bigger_than_enum_3_unpack_be(const uint8_t val[8]) {
  struct chip_reg_field_bigger_than_enum_3 r = {0};
  // TINY_ENUM_3 @ reg_field_bigger_than_enum_3[63:0]:
  r.tiny_enum_3 = (((uint64_t)(val[0] & 0xFFU)) << 56) | (((uint64_t)(val[1] & 0xFFU)) << 48) | (((uint64_t)(val[2] & 0xFFU)) << 40) | (((uint64_t)(val[3] & 0xFFU)) << 32) | (((uint64_t)(val[4] & 0xFFU)) << 24) | (((uint64_t)(val[5] & 0xFFU)) << 16) | (((uint64_t)(val[6] & 0xFFU)) << 8) | ((uint64_t)(val[7] & 0xFFU));
  return r;
}

/**
 * @brief Validate struct
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 * Confirms that all enums are valid, and all values fit into respective fields
 */
static inline int chip_validate_reg_field_bigger_than_enum_3(const struct chip_reg_field_bigger_than_enum_3 *r) {
  if (!(CHIP_IS_VALID_TINY_ENUM_3(r->tiny_enum_3))) return 1;
  return 0;
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_reg_field_bigger_than_enum_3_try_unpack_le(const uint8_t val[8], struct chip_reg_field_bigger_than_enum_3 *r) {
  *r = chip_reg_field_bigger_than_enum_3_unpack_le(val);
  return chip_validate_reg_field_bigger_than_enum_3(r);
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_reg_field_bigger_than_enum_3_try_unpack_be(const uint8_t val[8], struct chip_reg_field_bigger_than_enum_3 *r) {
  *r = chip_reg_field_bigger_than_enum_3_unpack_be(val);
  return chip_validate_reg_field_bigger_than_enum_3(r);
}

//===----------------------------------------------------------------------===//
// REG_FIELD_HUGE_ENUM Register
//===----------------------------------------------------------------------===//
// Fields:
//  - [63:0] HUGE_ENUM (enum HUGE_ENUM)

#define CHIP_REG_FIELD_HUGE_ENUM_ADDRESS (0x21U) //!< REG_FIELD_HUGE_ENUM register address

// Register-specific enums and sub-layouts:

/** @name HUGE_ENUM */
///@{
#define CHIP_HUGE_ENUM_F0 (0x0U)
#define CHIP_HUGE_ENUM_F1 (0xFFFFFFFFFFFFFFFFU)
///@}

// Register Layout Struct:

/** @note use pack/unpack functions for conversion to/from packed binary value */
struct chip_reg_field_huge_enum {
  uint64_t huge_enum;
};

// Enum validation functions:

/**
 * @brief Check if a numeric value is a valid @ref HUGE_ENUM.
 * @returns bool (true/false)
 */
#define CHIP_IS_VALID_HUGE_ENUM(_VAL_) (                                                           \\
  ((_VAL_) == 0x0U) ? true :                                                                       \\
  ((_VAL_) == 0xFFFFFFFFFFFFFFFFU) ? true :                                                        \\
  false )

// Layout struct conversion functions:

/**
 * @brief Convert @ref struct chip_reg_field_huge_enum struct to packed little-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_reg_field_huge_enum_pack_le(const struct chip_reg_field_huge_enum *r, uint8_t val[8]) {
  // HUGE_ENUM @ reg_field_huge_enum[63:0]:
  val[0] &= (uint8_t)~0xFFU;
  val[0] |= (uint8_t)(((uint8_t)r->huge_enum) & 0xFFU);
  val[1] &= (uint8_t)~0xFFU;
  val[1] |= (uint8_t)(((uint8_t)(r->huge_enum >> 8)) & 0xFFU);
  val[2] &= (uint8_t)~0xFFU;
  val[2] |= (uint8_t)(((uint8_t)(r->huge_enum >> 16)) & 0xFFU);
  val[3] &= (uint8_t)~0xFFU;
  val[3] |= (uint8_t)(((uint8_t)(r->huge_enum >> 24)) & 0xFFU);
  val[4] &= (uint8_t)~0xFFU;
  val[4] |= (uint8_t)(((uint8_t)(r->huge_enum >> 32)) & 0xFFU);
  val[5] &= (uint8_t)~0xFFU;
  val[5] |= (uint8_t)(((uint8_t)(r->huge_enum >> 40)) & 0xFFU);
  val[6] &= (uint8_t)~0xFFU;
  val[6] |= (uint8_t)(((uint8_t)(r->huge_enum >> 48)) & 0xFFU);
  val[7] &= (uint8_t)~0xFFU;
  val[7] |= (uint8_t)(((uint8_t)(r->huge_enum >> 56)) & 0xFFU);
}

/**
 * @brief Convert @ref struct chip_reg_field_huge_enum struct to packed big-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_reg_field_huge_enum_pack_be(const struct chip_reg_field_huge_enum *r, uint8_t val[8]) {
  // HUGE_ENUM @ reg_field_huge_enum[63:0]:
  val[0] &= (uint8_t)~0xFFU;
  val[0] |= (uint8_t)(((uint8_t)(r->huge_enum >> 56)) & 0xFFU);
  val[1] &= (uint8_t)~0xFFU;
  val[1] |= (uint8_t)(((uint8_t)(r->huge_enum >> 48)) & 0xFFU);
  val[2] &= (uint8_t)~0xFFU;
  val[2] |= (uint8_t)(((uint8_t)(r->huge_enum >> 40)) & 0xFFU);
  val[3] &= (uint8_t)~0xFFU;
  val[3] |= (uint8_t)(((uint8_t)(r->huge_enum >> 32)) & 0xFFU);
  val[4] &= (uint8_t)~0xFFU;
  val[4] |= (uint8_t)(((uint8_t)(r->huge_enum >> 24)) & 0xFFU);
  val[5] &= (uint8_t)~0xFFU;
  val[5] |= (uint8_t)(((uint8_t)(r->huge_enum >> 16)) & 0xFFU);
  val[6] &= (uint8_t)~0xFFU;
  val[6] |= (uint8_t)(((uint8_t)(r->huge_enum >> 8)) & 0xFFU);
  val[7] &= (uint8_t)~0xFFU;
  val[7] |= (uint8_t)(((uint8_t)r->huge_enum) & 0xFFU);
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_reg_field_huge_enum chip_reg_field_huge_enum_unpack_le(const uint8_t val[8]) {
  struct chip_reg_field_huge_enum r = {0};
  // HUGE_ENUM @ reg_field_huge_enum[63:0]:
  r.huge_enum = ((uint64_t)(val[0] & 0xFFU)) | (((uint64_t)(val[1] & 0xFFU)) << 8) | (((uint64_t)(val[2] & 0xFFU)) << 16) | (((uint64_t)(val[3] & 0xFFU)) << 24) | (((uint64_t)(val[4] & 0xFFU)) << 32) | (((uint64_t)(val[5] & 0xFFU)) << 40) | (((uint64_t)(val[6] & 0xFFU)) << 48) | (((uint64_t)(val[7] & 0xFFU)) << 56);
  return r;
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_reg_field_huge_enum chip_reg_field_huge_enum_unpack_be(const uint8_t val[8]) {
  struct chip_reg_field_huge_enum r = {0};
  // HUGE_ENUM @ reg_field_huge_enum[63:0]:
  r.huge_enum = (((uint64_t)(val[0] & 0xFFU)) << 56) | (((uint64_t)(val[1] & 0xFFU)) << 48) | (((uint64_t)(val[2] & 0xFFU)) << 40) | (((uint64_t)(val[3] & 0xFFU)) << 32) | (((uint64_t)(val[4] & 0xFFU)) << 24) | (((uint64_t)(val[5] & 0xFFU)) << 16) | (((uint64_t)(val[6] & 0xFFU)) << 8) | ((uint64_t)(val[7] & 0xFFU));
  return r;
}

/**
 * @brief Validate struct
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 * Confirms that all enums are valid, and all values fit into respective fields
 */
static inline int chip_validate_reg_field_huge_enum(const struct chip_reg_field_huge_enum *r) {
  if (!(CHIP_IS_VALID_HUGE_ENUM(r->huge_enum))) return 1;
  return 0;
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_reg_field_huge_enum_try_unpack_le(const uint8_t val[8], struct chip_reg_field_huge_enum *r) {
  *r = chip_reg_field_huge_enum_unpack_le(val);
  return chip_validate_reg_field_huge_enum(r);
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_reg_field_huge_enum_try_unpack_be(const uint8_t val[8], struct chip_reg_field_huge_enum *r) {
  *r = chip_reg_field_huge_enum_unpack_be(val);
  return chip_validate_reg_field_huge_enum(r);
}

//===----------------------------------------------------------------------===//
// REG_FIXED_ACROSS_BYTES Register
//===----------------------------------------------------------------------===//
// Fields:
//  - [9:6] F1 (fixed: 0x9)

#define CHIP_REG_FIXED_ACROSS_BYTES_ADDRESS (0x12U) //!< REG_FIXED_ACROSS_BYTES register address

// Register Layout Struct:

/** @note use pack/unpack functions for conversion to/from packed binary value */
struct chip_reg_fixed_across_bytes {
  int dummy; // Register contains no variable fields.
};

// Layout struct conversion functions:

/**
 * @brief Convert @ref struct chip_reg_fixed_across_bytes struct to packed little-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_reg_fixed_across_bytes_pack_le(const struct chip_reg_fixed_across_bytes *r, uint8_t val[2]) {
  // F1 @ reg_fixed_across_bytes[9:6]:
  val[0] &= (uint8_t)~0xC0U;
  val[0] |= (uint8_t)0x40; // Fixed value.
  val[1] &= (uint8_t)~0x3U;
  val[1] |= (uint8_t)0x2; // Fixed value.
  (void)r;
}

/**
 * @brief Convert @ref struct chip_reg_fixed_across_bytes struct to packed big-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_reg_fixed_across_bytes_pack_be(const struct chip_reg_fixed_across_bytes *r, uint8_t val[2]) {
  // F1 @ reg_fixed_across_bytes[9:6]:
  val[0] &= (uint8_t)~0x3U;
  val[0] |= (uint8_t)0x2; // Fixed value.
  val[1] &= (uint8_t)~0xC0U;
  val[1] |= (uint8_t)0x40; // Fixed value.
  (void)r;
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_reg_fixed_across_bytes chip_reg_fixed_across_bytes_unpack_le(const uint8_t val[2]) {
  struct chip_reg_fixed_across_bytes r = {0};
  (void)val;
  (void)r;
  return r;
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_reg_fixed_across_bytes chip_reg_fixed_across_bytes_unpack_be(const uint8_t val[2]) {
  struct chip_reg_fixed_across_bytes r = {0};
  (void)val;
  (void)r;
  return r;
}

/**
 * @brief Validate struct
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 * Confirms that all enums are valid, and all values fit into respective fields
 */
static inline int chip_validate_reg_fixed_across_bytes(const struct chip_reg_fixed_across_bytes *r) {
  (void)r;
  return 0;
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_reg_fixed_across_bytes_try_unpack_le(const uint8_t val[2], struct chip_reg_fixed_across_bytes *r) {
  *r = chip_reg_fixed_across_bytes_unpack_le(val);
  return chip_validate_reg_fixed_across_bytes(r);
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_reg_fixed_across_bytes_try_unpack_be(const uint8_t val[2], struct chip_reg_fixed_across_bytes *r) {
  *r = chip_reg_fixed_across_bytes_unpack_be(val);
  return chip_validate_reg_fixed_across_bytes(r);
}

//===----------------------------------------------------------------------===//
// REG_LAYOUT_FIELD Register
//===----------------------------------------------------------------------===//
// Fields:
//  - [15:0] LAYOUT_FIELD (layout LAYOUT_FIELD)
//    - [0] LAYOUT_FIELD.F1 (uint)
//    - [9:2] LAYOUT_FIELD.F2 (layout F2)
//      - [9:2] LAYOUT_FIELD.F2.F22 (uint)
//    - [13] LAYOUT_FIELD.F3 (layout F3)

#define CHIP_REG_LAYOUT_FIELD_ADDRESS  (0x20U)       //!< REG_LAYOUT_FIELD register address
#define CHIP_REG_LAYOUT_FIELD_RESET_LE {0x3U, 0xFDU} //!< REG_LAYOUT_FIELD register reset value
#define CHIP_REG_LAYOUT_FIELD_RESET_BE {0xFDU, 0x3U} //!< REG_LAYOUT_FIELD register reset value

// Register-specific enums and sub-layouts:

/** @note use pack/unpack functions for conversion to/from packed binary value */
struct chip_f2 {
  uint8_t f22;
};

/** @note use pack/unpack functions for conversion to/from packed binary value */
struct chip_f3 {
  int dummy; // Register contains no variable fields.
};

/** @note use pack/unpack functions for conversion to/from packed binary value */
struct chip_layout_field {
  uint8_t f1;
  struct chip_f2 f2;
  struct chip_f3 f3;
};

// Register Layout Struct:

/** @note use pack/unpack functions for conversion to/from packed binary value */
struct chip_reg_layout_field {
  struct chip_layout_field layout_field;
};

// Layout struct conversion functions:

/**
 * @brief Convert @ref struct chip_f2 struct to packed little-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_f2_pack_le(const struct chip_f2 *r, uint8_t val[1]) {
  // F22 @ f2[7:0]:
  val[0] &= (uint8_t)~0xFFU;
  val[0] |= (uint8_t)(((uint8_t)r->f22) & 0xFFU);
}

/**
 * @brief Convert @ref struct chip_f2 struct to packed big-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_f2_pack_be(const struct chip_f2 *r, uint8_t val[1]) {
  // F22 @ f2[7:0]:
  val[0] &= (uint8_t)~0xFFU;
  val[0] |= (uint8_t)(((uint8_t)r->f22) & 0xFFU);
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_f2 chip_f2_unpack_le(const uint8_t val[1]) {
  struct chip_f2 r = {0};
  // F22 @ f2[7:0]:
  r.f22 = (uint8_t)((val[0] & 0xFFU));
  return r;
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_f2 chip_f2_unpack_be(const uint8_t val[1]) {
  struct chip_f2 r = {0};
  // F22 @ f2[7:0]:
  r.f22 = (uint8_t)((val[0] & 0xFFU));
  return r;
}

/**
 * @brief Validate struct
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 * Confirms that all enums are valid, and all values fit into respective fields
 */
static inline int chip_validate_f2(const struct chip_f2 *r) {
  if ((r->f22 & ~(uint8_t)0xFF) != 0) return 1;
  return 0;
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_f2_try_unpack_le(const uint8_t val[1], struct chip_f2 *r) {
  *r = chip_f2_unpack_le(val);
  return chip_validate_f2(r);
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_f2_try_unpack_be(const uint8_t val[1], struct chip_f2 *r) {
  *r = chip_f2_unpack_be(val);
  return chip_validate_f2(r);
}

/**
 * @brief Convert @ref struct chip_f3 struct to packed little-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_f3_pack_le(const struct chip_f3 *r, uint8_t val[1]) {
  (void)val;
  (void)r;
}

/**
 * @brief Convert @ref struct chip_f3 struct to packed big-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_f3_pack_be(const struct chip_f3 *r, uint8_t val[1]) {
  (void)val;
  (void)r;
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_f3 chip_f3_unpack_le(const uint8_t val[1]) {
  struct chip_f3 r = {0};
  (void)val;
  (void)r;
  return r;
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_f3 chip_f3_unpack_be(const uint8_t val[1]) {
  struct chip_f3 r = {0};
  (void)val;
  (void)r;
  return r;
}

/**
 * @brief Validate struct
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 * Confirms that all enums are valid, and all values fit into respective fields
 */
static inline int chip_validate_f3(const struct chip_f3 *r) {
  (void)r;
  return 0;
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_f3_try_unpack_le(const uint8_t val[1], struct chip_f3 *r) {
  *r = chip_f3_unpack_le(val);
  return chip_validate_f3(r);
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_f3_try_unpack_be(const uint8_t val[1], struct chip_f3 *r) {
  *r = chip_f3_unpack_be(val);
  return chip_validate_f3(r);
}

/**
 * @brief Convert @ref struct chip_layout_field struct to packed little-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_layout_field_pack_le(const struct chip_layout_field *r, uint8_t val[2]) {
  // F1 @ layout_field[0]:
  val[0] &= (uint8_t)~0x1U;
  val[0] |= (uint8_t)(((uint8_t)r->f1) & 0x1U);
  // F2 @ layout_field[9:2]:
  uint8_t f2[1] = {0};
  chip_f2_pack_le(&r->f2, f2);
  val[0] &= (uint8_t)~0xFCU;
  val[0] |= (uint8_t)((uint8_t)(f2[0] << 2) & 0xFCU);
  val[1] &= (uint8_t)~0x3U;
  val[1] |= (uint8_t)((uint8_t)(f2[0] >> 6) & 0x3U);
  // F3 @ layout_field[13]:
  uint8_t f3[1] = {0};
  chip_f3_pack_le(&r->f3, f3);
}

/**
 * @brief Convert @ref struct chip_layout_field struct to packed big-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_layout_field_pack_be(const struct chip_layout_field *r, uint8_t val[2]) {
  // F1 @ layout_field[0]:
  val[1] &= (uint8_t)~0x1U;
  val[1] |= (uint8_t)(((uint8_t)r->f1) & 0x1U);
  // F2 @ layout_field[9:2]:
  uint8_t f2[1] = {0};
  chip_f2_pack_be(&r->f2, f2);
  val[0] &= (uint8_t)~0x3U;
  val[0] |= (uint8_t)((uint8_t)(f2[0] >> 6) & 0x3U);
  val[1] &= (uint8_t)~0xFCU;
  val[1] |= (uint8_t)((uint8_t)(f2[0] << 2) & 0xFCU);
  // F3 @ layout_field[13]:
  uint8_t f3[1] = {0};
  chip_f3_pack_be(&r->f3, f3);
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_layout_field chip_layout_field_unpack_le(const uint8_t val[2]) {
  struct chip_layout_field r = {0};
  // F1 @ layout_field[0]:
  r.f1 = (uint8_t)((val[0] & 0x1U));
  // F2 @ layout_field[9:2]:
  uint8_t f2[1] = {0};
  f2[0] |= (uint8_t)((val[0] & 0xFCU) >> 2);
  f2[0] |= (uint8_t)((val[1] & 0x3U) << 6);
  r.f2 = chip_f2_unpack_le(f2);
  // F3 @ layout_field[13]:
  uint8_t f3[1] = {0};
  r.f3 = chip_f3_unpack_le(f3);
  return r;
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_layout_field chip_layout_field_unpack_be(const uint8_t val[2]) {
  struct chip_layout_field r = {0};
  // F1 @ layout_field[0]:
  r.f1 = (uint8_t)((val[1] & 0x1U));
  // F2 @ layout_field[9:2]:
  uint8_t f2[1] = {0};
  f2[0] |= (uint8_t)((val[0] & 0x3U) << 6);
  f2[0] |= (uint8_t)((val[1] & 0xFCU) >> 2);
  r.f2 = chip_f2_unpack_be(f2);
  // F3 @ layout_field[13]:
  uint8_t f3[1] = {0};
  r.f3 = chip_f3_unpack_be(f3);
  return r;
}

/**
 * @brief Validate struct
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 * Confirms that all enums are valid, and all values fit into respective fields
 */
static inline int chip_validate_layout_field(const struct chip_layout_field *r) {
  if ((r->f1 & ~(uint8_t)0x1) != 0) return 1;
  if (chip_validate_f2(&r->f2)) return 3;
  if (chip_validate_f3(&r->f3)) return 14;
  return 0;
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_layout_field_try_unpack_le(const uint8_t val[2], struct chip_layout_field *r) {
  *r = chip_layout_field_unpack_le(val);
  return chip_validate_layout_field(r);
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_layout_field_try_unpack_be(const uint8_t val[2], struct chip_layout_field *r) {
  *r = chip_layout_field_unpack_be(val);
  return chip_validate_layout_field(r);
}

/**
 * @brief Convert @ref struct chip_reg_layout_field struct to packed little-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_reg_layout_field_pack_le(const struct chip_reg_layout_field *r, uint8_t val[2]) {
  // LAYOUT_FIELD @ reg_layout_field[15:0]:
  uint8_t layout_field[2] = {0};
  chip_layout_field_pack_le(&r->layout_field, layout_field);
  val[0] &= (uint8_t)~0xFDU;
  val[0] |= (uint8_t)((uint8_t)layout_field[0] & 0xFDU);
  val[1] &= (uint8_t)~0x23U;
  val[1] |= (uint8_t)((uint8_t)layout_field[1] & 0x23U);
}

/**
 * @brief Convert @ref struct chip_reg_layout_field struct to packed big-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_reg_layout_field_pack_be(const struct chip_reg_layout_field *r, uint8_t val[2]) {
  // LAYOUT_FIELD @ reg_layout_field[15:0]:
  uint8_t layout_field[2] = {0};
  chip_layout_field_pack_be(&r->layout_field, layout_field);
  val[0] &= (uint8_t)~0x23U;
  val[0] |= (uint8_t)((uint8_t)layout_field[0] & 0x23U);
  val[1] &= (uint8_t)~0xFDU;
  val[1] |= (uint8_t)((uint8_t)layout_field[1] & 0xFDU);
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_reg_layout_field chip_reg_layout_field_unpack_le(const uint8_t val[2]) {
  struct chip_reg_layout_field r = {0};
  // LAYOUT_FIELD @ reg_layout_field[15:0]:
  uint8_t layout_field[2] = {0};
  layout_field[0] |= (uint8_t)((val[0] & 0xFDU));
  layout_field[1] |= (uint8_t)((val[1] & 0x23U));
  r.layout_field = chip_layout_field_unpack_le(layout_field);
  return r;
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_reg_layout_field chip_reg_layout_field_unpack_be(const uint8_t val[2]) {
  struct chip_reg_layout_field r = {0};
  // LAYOUT_FIELD @ reg_layout_field[15:0]:
  uint8_t layout_field[2] = {0};
  layout_field[0] |= (uint8_t)((val[0] & 0x23U));
  layout_field[1] |= (uint8_t)((val[1] & 0xFDU));
  r.layout_field = chip_layout_field_unpack_be(layout_field);
  return r;
}

/**
 * @brief Validate struct
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 * Confirms that all enums are valid, and all values fit into respective fields
 */
static inline int chip_validate_reg_layout_field(const struct chip_reg_layout_field *r) {
  if (chip_validate_layout_field(&r->layout_field)) return 1;
  return 0;
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_reg_layout_field_try_unpack_le(const uint8_t val[2], struct chip_reg_layout_field *r) {
  *r = chip_reg_layout_field_unpack_le(val);
  return chip_validate_reg_layout_field(r);
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_reg_layout_field_try_unpack_be(const uint8_t val[2], struct chip_reg_layout_field *r) {
  *r = chip_reg_layout_field_unpack_be(val);
  return chip_validate_reg_layout_field(r);
}

//===----------------------------------------------------------------------===//
// REG_NESTED_ONLY_FIXED Register
//===----------------------------------------------------------------------===//
// Fields:
//  - [7:0] LAYOUT_FIELD_1 (layout LAYOUT_FIELD_1)
//    - [7:0] LAYOUT_FIELD_1.LAYOUT_FIELD_2 (fixed: 0xab)

#define CHIP_REG_NESTED_ONLY_FIXED_ADDRESS (0x20U) //!< REG_NESTED_ONLY_FIXED register address

// Register-specific enums and sub-layouts:

/** @note use pack/unpack functions for conversion to/from packed binary value */
struct chip_layout_field_1 {
  int dummy; // Register contains no variable fields.
};

// Register Layout Struct:

/** @note use pack/unpack functions for conversion to/from packed binary value */
struct chip_reg_nested_only_fixed {
  struct chip_layout_field_1 layout_field_1;
};

// Layout struct conversion functions:

/**
 * @brief Convert @ref struct chip_layout_field_1 struct to packed little-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_layout_field_1_pack_le(const struct chip_layout_field_1 *r, uint8_t val[1]) {
  // LAYOUT_FIELD_2 @ layout_field_1[7:0]:
  val[0] &= (uint8_t)~0xFFU;
  val[0] |= (uint8_t)0xab; // Fixed value.
  (void)r;
}

/**
 * @brief Convert @ref struct chip_layout_field_1 struct to packed big-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_layout_field_1_pack_be(const struct chip_layout_field_1 *r, uint8_t val[1]) {
  // LAYOUT_FIELD_2 @ layout_field_1[7:0]:
  val[0] &= (uint8_t)~0xFFU;
  val[0] |= (uint8_t)0xab; // Fixed value.
  (void)r;
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_layout_field_1 chip_layout_field_1_unpack_le(const uint8_t val[1]) {
  struct chip_layout_field_1 r = {0};
  (void)val;
  (void)r;
  return r;
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_layout_field_1 chip_layout_field_1_unpack_be(const uint8_t val[1]) {
  struct chip_layout_field_1 r = {0};
  (void)val;
  (void)r;
  return r;
}

/**
 * @brief Validate struct
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 * Confirms that all enums are valid, and all values fit into respective fields
 */
static inline int chip_validate_layout_field_1(const struct chip_layout_field_1 *r) {
  (void)r;
  return 0;
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_layout_field_1_try_unpack_le(const uint8_t val[1], struct chip_layout_field_1 *r) {
  *r = chip_layout_field_1_unpack_le(val);
  return chip_validate_layout_field_1(r);
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_layout_field_1_try_unpack_be(const uint8_t val[1], struct chip_layout_field_1 *r) {
  *r = chip_layout_field_1_unpack_be(val);
  return chip_validate_layout_field_1(r);
}

/**
 * @brief Convert @ref struct chip_reg_nested_only_fixed struct to packed little-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_reg_nested_only_fixed_pack_le(const struct chip_reg_nested_only_fixed *r, uint8_t val[1]) {
  // LAYOUT_FIELD_1 @ reg_nested_only_fixed[7:0]:
  uint8_t layout_field_1[1] = {0};
  chip_layout_field_1_pack_le(&r->layout_field_1, layout_field_1);
  val[0] &= (uint8_t)~0xFFU;
  val[0] |= (uint8_t)((uint8_t)layout_field_1[0] & 0xFFU);
}

/**
 * @brief Convert @ref struct chip_reg_nested_only_fixed struct to packed big-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_reg_nested_only_fixed_pack_be(const struct chip_reg_nested_only_fixed *r, uint8_t val[1]) {
  // LAYOUT_FIELD_1 @ reg_nested_only_fixed[7:0]:
  uint8_t layout_field_1[1] = {0};
  chip_layout_field_1_pack_be(&r->layout_field_1, layout_field_1);
  val[0] &= (uint8_t)~0xFFU;
  val[0] |= (uint8_t)((uint8_t)layout_field_1[0] & 0xFFU);
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_reg_nested_only_fixed chip_reg_nested_only_fixed_unpack_le(const uint8_t val[1]) {
  struct chip_reg_nested_only_fixed r = {0};
  // LAYOUT_FIELD_1 @ reg_nested_only_fixed[7:0]:
  uint8_t layout_field_1[1] = {0};
  layout_field_1[0] |= (uint8_t)((val[0] & 0xFFU));
  r.layout_field_1 = chip_layout_field_1_unpack_le(layout_field_1);
  return r;
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_reg_nested_only_fixed chip_reg_nested_only_fixed_unpack_be(const uint8_t val[1]) {
  struct chip_reg_nested_only_fixed r = {0};
  // LAYOUT_FIELD_1 @ reg_nested_only_fixed[7:0]:
  uint8_t layout_field_1[1] = {0};
  layout_field_1[0] |= (uint8_t)((val[0] & 0xFFU));
  r.layout_field_1 = chip_layout_field_1_unpack_be(layout_field_1);
  return r;
}

/**
 * @brief Validate struct
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 * Confirms that all enums are valid, and all values fit into respective fields
 */
static inline int chip_validate_reg_nested_only_fixed(const struct chip_reg_nested_only_fixed *r) {
  if (chip_validate_layout_field_1(&r->layout_field_1)) return 1;
  return 0;
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_reg_nested_only_fixed_try_unpack_le(const uint8_t val[1], struct chip_reg_nested_only_fixed *r) {
  *r = chip_reg_nested_only_fixed_unpack_le(val);
  return chip_validate_reg_nested_only_fixed(r);
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_reg_nested_only_fixed_try_unpack_be(const uint8_t val[1], struct chip_reg_nested_only_fixed *r) {
  *r = chip_reg_nested_only_fixed_unpack_be(val);
  return chip_validate_reg_nested_only_fixed(r);
}

//===----------------------------------------------------------------------===//
// REG_SHARED_LAYOUT Register
//===----------------------------------------------------------------------===//
// Fields:
//  - [4:1] SHARED_FIELD1 (uint)
//  - [10] SHARED_FIELD2 (enum SHARED_FIELD2)

#define CHIP_REG_SHARED_LAYOUT_ADDRESS (0x21U) //!< REG_SHARED_LAYOUT register address

// Register uses the chip_basic_shared_layout struct and conversion funcs defined above.

//===----------------------------------------------------------------------===//
// REG_SHARED_LAYOUT_BASIC_1 Register
//===----------------------------------------------------------------------===//
// Fields:
//  - [4:1] SHARED_FIELD1 (uint)
//  - [10] SHARED_FIELD2 (enum SHARED_FIELD2)

#define CHIP_REG_SHARED_LAYOUT_BASIC_1_ADDRESS  (0x10U)      //!< REG_SHARED_LAYOUT_BASIC_1 register address
#define CHIP_REG_SHARED_LAYOUT_BASIC_1_RESET_LE {0x0U, 0x0U} //!< REG_SHARED_LAYOUT_BASIC_1 register reset value
#define CHIP_REG_SHARED_LAYOUT_BASIC_1_RESET_BE {0x0U, 0x0U} //!< REG_SHARED_LAYOUT_BASIC_1 register reset value

// Register uses the chip_basic_shared_layout struct and conversion funcs defined above.

//===----------------------------------------------------------------------===//
// REG_SHARED_LAYOUT_BASIC_2 Register
//===----------------------------------------------------------------------===//
// Fields:
//  - [4:1] SHARED_FIELD1 (uint)
//  - [10] SHARED_FIELD2 (enum SHARED_FIELD2)

#define CHIP_REG_SHARED_LAYOUT_BASIC_2_ADDRESS (0x10U) //!< REG_SHARED_LAYOUT_BASIC_2 register address

// Register uses the chip_basic_shared_layout struct and conversion funcs defined above.

//===----------------------------------------------------------------------===//
// BLOCK Register Block
//===----------------------------------------------------------------------===//
//
// Contains registers:
// - [0x00] BLOCK_MEMBER_A
// - [0x01] BLOCK_MEMBER_B
//
// Instances:
// - [0x16] BLOCK1
// - [0x32] BLOCK2

// Contained registers:
#define CHIP_BLOCK_MEMBER_A_OFFSET (0x0U) //!< Offset of BLOCK_MEMBER_A register from BLOCK block start
#define CHIP_BLOCK_MEMBER_B_OFFSET (0x1U) //!< Offset of BLOCK_MEMBER_B register from BLOCK block start

// Instances:
#define CHIP_BLOCK_INSTANCE_BLOCK1 (0x10U) //!< Start of BLOCK instance BLOCK1
#define CHIP_BLOCK_INSTANCE_BLOCK2 (0x20U) //!< Start of BLOCK instance BLOCK2

// ---- BLOCK_MEMBER_A Register Block Member  ----------------------------------
// Fields:
//  - [7:0] VAL (uint)

#define CHIP_BLOCK1_MEMBER_A_ADDRESS  (0x10U) //!< BLOCK1_MEMBER_A register address
#define CHIP_BLOCK1_MEMBER_A_RESET_LE {0x1BU} //!< BLOCK1_MEMBER_A register reset value
#define CHIP_BLOCK1_MEMBER_A_RESET_BE {0x1BU} //!< BLOCK1_MEMBER_A register reset value
#define CHIP_BLOCK2_MEMBER_A_ADDRESS  (0x20U) //!< BLOCK2_MEMBER_A register address
#define CHIP_BLOCK2_MEMBER_A_RESET_LE {0x1BU} //!< BLOCK2_MEMBER_A register reset value
#define CHIP_BLOCK2_MEMBER_A_RESET_BE {0x1BU} //!< BLOCK2_MEMBER_A register reset value

// Register Layout Struct:

/** @note use pack/unpack functions for conversion to/from packed binary value */
struct chip_block_member_a {
  uint8_t val;
};

// Layout struct conversion functions:

/**
 * @brief Convert @ref struct chip_block_member_a struct to packed little-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_block_member_a_pack_le(const struct chip_block_member_a *r, uint8_t val[1]) {
  // VAL @ block_member_a[7:0]:
  val[0] &= (uint8_t)~0xFFU;
  val[0] |= (uint8_t)(((uint8_t)r->val) & 0xFFU);
}

/**
 * @brief Convert @ref struct chip_block_member_a struct to packed big-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_block_member_a_pack_be(const struct chip_block_member_a *r, uint8_t val[1]) {
  // VAL @ block_member_a[7:0]:
  val[0] &= (uint8_t)~0xFFU;
  val[0] |= (uint8_t)(((uint8_t)r->val) & 0xFFU);
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_block_member_a chip_block_member_a_unpack_le(const uint8_t val[1]) {
  struct chip_block_member_a r = {0};
  // VAL @ block_member_a[7:0]:
  r.val = (uint8_t)((val[0] & 0xFFU));
  return r;
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_block_member_a chip_block_member_a_unpack_be(const uint8_t val[1]) {
  struct chip_block_member_a r = {0};
  // VAL @ block_member_a[7:0]:
  r.val = (uint8_t)((val[0] & 0xFFU));
  return r;
}

/**
 * @brief Validate struct
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 * Confirms that all enums are valid, and all values fit into respective fields
 */
static inline int chip_validate_block_member_a(const struct chip_block_member_a *r) {
  if ((r->val & ~(uint8_t)0xFF) != 0) return 1;
  return 0;
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_block_member_a_try_unpack_le(const uint8_t val[1], struct chip_block_member_a *r) {
  *r = chip_block_member_a_unpack_le(val);
  return chip_validate_block_member_a(r);
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_block_member_a_try_unpack_be(const uint8_t val[1], struct chip_block_member_a *r) {
  *r = chip_block_member_a_unpack_be(val);
  return chip_validate_block_member_a(r);
}

// ---- BLOCK_MEMBER_B Register Block Member  ----------------------------------
// Fields:
//  - [6:0] VAL (uint)

#define CHIP_BLOCK1_MEMBER_B_ADDRESS (0x11U) //!< BLOCK1_MEMBER_B register address
#define CHIP_BLOCK2_MEMBER_B_ADDRESS (0x21U) //!< BLOCK2_MEMBER_B register address

// Register Layout Struct:

/** @note use pack/unpack functions for conversion to/from packed binary value */
struct chip_block_member_b {
  uint8_t val;
};

// Layout struct conversion functions:

/**
 * @brief Convert @ref struct chip_block_member_b struct to packed little-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_block_member_b_pack_le(const struct chip_block_member_b *r, uint8_t val[1]) {
  // VAL @ block_member_b[6:0]:
  val[0] &= (uint8_t)~0x7FU;
  val[0] |= (uint8_t)(((uint8_t)r->val) & 0x7FU);
}

/**
 * @brief Convert @ref struct chip_block_member_b struct to packed big-endian value.
 * @note use pack/unpack functions for conversion to/from packed binary value
 */
static inline void chip_block_member_b_pack_be(const struct chip_block_member_b *r, uint8_t val[1]) {
  // VAL @ block_member_b[6:0]:
  val[0] &= (uint8_t)~0x7FU;
  val[0] |= (uint8_t)(((uint8_t)r->val) & 0x7FU);
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_block_member_b chip_block_member_b_unpack_le(const uint8_t val[1]) {
  struct chip_block_member_b r = {0};
  // VAL @ block_member_b[6:0]:
  r.val = (uint8_t)((val[0] & 0x7FU));
  return r;
}

/** @brief Convert packed {endian} binary value to struct. */
static inline struct chip_block_member_b chip_block_member_b_unpack_be(const uint8_t val[1]) {
  struct chip_block_member_b r = {0};
  // VAL @ block_member_b[6:0]:
  r.val = (uint8_t)((val[0] & 0x7FU));
  return r;
}

/**
 * @brief Validate struct
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 * Confirms that all enums are valid, and all values fit into respective fields
 */
static inline int chip_validate_block_member_b(const struct chip_block_member_b *r) {
  if ((r->val & ~(uint8_t)0x7F) != 0) return 1;
  return 0;
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_block_member_b_try_unpack_le(const uint8_t val[1], struct chip_block_member_b *r) {
  *r = chip_block_member_b_unpack_le(val);
  return chip_validate_block_member_b(r);
}

/**
 * @brief Attempt to convert packed {endian} binary value to struct.
 * @returns 0 if valid.
 * @returns the position of the first invalid field if invalid.
 */
static inline int chip_block_member_b_try_unpack_be(const uint8_t val[1], struct chip_block_member_b *r) {
  *r = chip_block_member_b_unpack_be(val);
  return chip_validate_block_member_b(r);
}
`;
