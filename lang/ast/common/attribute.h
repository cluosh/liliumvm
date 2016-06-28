/*
 * Copyright (C) 2016  Michael Pucher (cluosh)
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 3 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
 */
#ifndef LANG_AST_COMMON_ATTRIBUTE_H_
#define LANG_AST_COMMON_ATTRIBUTE_H_

#include <cstdint>
#include <string>
#include <vector>
#include <unordered_map>
#include <utility>

#include "vm/types/types.h"
#include "vm/bytecode/generator.h"

namespace AST {

/**
 * Structure for storing information passed down during generation and
 * attributation.
 */
struct AttribInfo {
  VM::FuncAddr func_addr;
  VM::ConstPool constants;
  std::uint64_t code_counter;
  std::uint8_t next_reg;
};

}  // namespace AST

#endif  // LANG_AST_COMMON_ATTRIBUTE_H_
