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
#include <fstream>

#include "cc/parser/controller.h"
#include "vm/bytecode/generator.h"

namespace Parser {

/**
 * @brief Clean up the controller
 */
Controller::~Controller() {
  delete ast;
  delete parser;
  delete scanner;
}

/**
 * @brief Open a file-stream and parse the file
 * @param filename File to be parsed
 * @return False on error
 */
bool Controller::parse(std::string filename) {
  std::ifstream inp(filename);
  return inp.good() && parse(inp);
}

/**
 * @brief Use an opened stream for parsing
 * @param is Input stream to be parsed
 * @return False on error
 */
bool Controller::parse(const std::istream &is) {
  if (!is.good() || is.eof()) {
    return false;
  }

  // Allocate AST
  delete ast;
  ast = new (std::nothrow) AST::Program();
  if (ast == nullptr) {
    std::cerr << "Could not allocate AST structure\n";
    return false;
  }

  // Create scanner
  delete scanner;
  scanner = new (std::nothrow) Scanner(const_cast<std::istream *>(&is));
  if (scanner == nullptr) {
    std::cerr << "Could not allocate scanner\n";
    return false;
  }

  // Create parser
  delete parser;
  parser = new (std::nothrow) Parser(*scanner, *ast);
  if (parser == nullptr) {
    std::cerr << "Could not allocate parser\n";
    return false;
  }

  // Parse
  if (parser->parse() != 0) return false;

  // Attribute syntax tree
  ast->attribute_tree();

  // Generate code
  VM::ByteCode::Generator generator(std::cout);
  ast->generate_code(&generator);
  return true;
}

}  // namespace Parser