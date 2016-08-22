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
#ifndef LANG_AST_PROGRAM_H_
#define LANG_AST_PROGRAM_H_

#include <cstdint>
#include <list>
#include <ostream>
#include <string>

#include "cc/ast/expr/global_expr.h"
#include "vm/bytecode/generator.h"

namespace AST {

/**
 * The root node of a lilium program.
 */
class Program {
 private:
  std::list<GlobalExpr *> expr_list;
  AttribInfo attrib_info;

 public:
  ~Program();

  void add(GlobalExpr *expr);
  void attribute_tree();
  void generate_code(VM::ByteCode::Generator *generator);
};

}  // namespace AST

#endif  // LANG_AST_PROGRAM_H_