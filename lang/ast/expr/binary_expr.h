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
#ifndef LANG_AST_EXPR_BINARY_EXPR_H_
#define LANG_AST_EXPR_BINARY_EXPR_H_

#include <string>

#include "lang/ast/expr/expr.h"
#include "lang/ast/common/operators.h"
#include "vm/opcodes.h"

namespace AST {

/**
 * Syntax node corresponding to all binary expressions.
 */
class BinaryExpr: public Expr {
 private:
  Expr *fst = nullptr;
  Expr *snd = nullptr;
  BinaryOperator op = BINARY_COUNT;

  VM::OpCode pick_typed();

 public:
  BinaryExpr(Expr *fst, Expr *snd, BinaryOperator op, Expr *next);
  ~BinaryExpr();

  void attribute(AttribInfo *attrib_info) override;
  void generate_code(VM::Generator *generator,
                     AttribInfo *attrib_info) override;
  void set_symbols(SymbolTables *symbol_tables) override;
};

}  // namespace AST

#endif  // LANG_AST_EXPR_BINARY_EXPR_H_
