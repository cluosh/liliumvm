/// Code in this module deals with generating an instruction stream from
/// the AST provided by the parser. The instructions are being stored in
/// an appropriate data structure, this module is not concerned with
/// packing code into modules.
use std::collections::HashMap;
use std::convert::TryFrom;
use common::*;
use compiler::parser::{Expression, Expression::*};

/// Structure for performing optimizations
struct OptimizationInfo<'a> {
    func_name: &'a str,
    tail: bool,
}

/// Generate a module from the abstract syntax tree.
///
/// # Arguments
///
/// * `expressions` - All top level expressions (AST roots) generated by the parser
///
/// # Remarks
///
/// Function definitions are processed first and placed at the beginning of the
/// module, which allows for easier processing. The entry point of the module
/// points to the first top level expression being evaluated.
pub fn generate(expressions: &[Expression]) -> Module {
    let mut func: HashMap<String, u32> = HashMap::new();
    let vars: HashMap<String, (Type, Register)> = HashMap::new();
    let mut module = Module {
        functions: Vec::new(),
        constants: Vec::new(),
        entry_point: 0,
        code: Vec::new()
    };

    // Initial optimization info structure
    let oinfo = OptimizationInfo {
        func_name: "NONE",
        tail: false
    };

    // Process function definitions first
    let filtered = expressions.iter().filter(|&x| match *x {
        FunctionDefinition(_,_,_) => true,
        _ => false
    });
    for expr in filtered {
        generate_expression(expr, reg::VAL, &mut func, &vars, &mut module, &oinfo);
    }

    // Process top-level expressions to be evaluated
    module.entry_point = module.code.len() as u64;
    let filtered = expressions.iter().filter(|&x| match *x {
        FunctionDefinition(_,_,_) => false,
        _ => true
    });
    for expr in filtered {
        generate_expression(expr, reg::VAL, &mut func, &vars, &mut module, &oinfo);
    }

    // Always end with halt instruction
    module.code.push(Instruction {
        opcode: ops::HLT,
        target: 0,
        left: 0,
        right: 0
    });

    module
}

/// Generate instructions for an AST with expression as its root node.
///
/// # Arguments
///
/// * `expr` - Root expression of the AST
/// * `base` - Base register of the expression, return value is stored here
/// * `func` - Lookup table for function table entries
/// * `vars` - A variable assignment for all child expressions
/// * `module` - Module to be filled with constant/function/code storage
/// * `oinfo` - Information used for optimization
fn generate_expression(expr: &Expression,
                       base: u8,
                       func: &mut HashMap<String, u32>,
                       vars: &HashMap<String, (Type, Register)>,
                       module: &mut Module,
                       oinfo: &OptimizationInfo) {
    match *expr {
        Integer(i) => {
            expr_integer(i, base, module);
        }
        BinaryOp(ref op, ref left, ref right) => {
            let optimizations = OptimizationInfo {
                func_name: oinfo.func_name,
                tail: false
            };
            expr_binary(op, left, right, base, func, vars, module, &optimizations);
        }
        UnaryOp(ref op, ref left) => {
            let optimizations = OptimizationInfo {
                func_name: oinfo.func_name,
                tail: false
            };
            expr_unary(op, left, base, func, vars, module, &optimizations);
        }
        NullaryOp(ref op) => {
            expr_nullary(op, base, module);
        }
        Function(ref name, ref param) => {
            expr_call(name, param, base, func, vars, module, oinfo);
        }
        FunctionDefinition(ref name, ref param, ref body) => {
            let optimizations = OptimizationInfo {
                func_name: name,
                tail: true
            };
            expr_fundef(name, param, body, base, func, vars, module, &optimizations);
        }
        VariableAssignment(ref assignments, ref body) => {
            let optimizations = OptimizationInfo {
                func_name: oinfo.func_name,
                tail: false
            };
            expr_varass(assignments, body, base, func, vars, module, &optimizations);
        }
        Variable(ref name) => {
            expr_variable(name, base, vars, module);
        }
        Conditional(ref condition, ref yes, ref no) => {
            expr_conditional(condition, yes, no, base, func, vars, module, &oinfo);
        }
    }
}

/// Generate instructions for a constant integer node.
///
/// # Arguments
///
/// * `value` - 64-bit signed integer value
/// * `base` - Base register of the expression, return value is stored here
/// * `module` - Module to be filled with constant/function/code storage
///
/// # Remarks
///
/// This generation always tries to fit an integer into 16 bit, if this is not
/// possible, a constant table entry is being created
#[inline(always)]
fn expr_integer(value: i64,
                base: u8,
                module: &mut Module) {
    match i16::try_from(value) {
        Ok(value) => {
            let left = value as u8;
            let right = (value >> 8) as u8;

            module.code.push(Instruction {
                opcode: ops::LD,
                target: base,
                left,
                right
            });
        }
        Err(_) => {
            let len = module.constants.len();
            let len = u16::try_from(len)
                .expect("Reached maximum number of constants.");
            let left = len as u8;
            let right = (len >> 8) as u8;

            module.constants.push(value);
            module.code.push(Instruction {
                opcode: ops::LDB,
                target: base,
                left,
                right
            });
        }
    }
}

/// Generate instructions for a binary operation.
///
/// # Arguments
///
/// * `op` - Name of the operation
/// * `left` - Left operand
/// * `right` - Right operand
/// * `base` - Base register of the expression, return value is stored here
/// * `func` - Lookup table for function table entries
/// * `vars` - A variable assignment for all child expressions
/// * `module` - Module to be filled with constant/function/code storage
/// * `oinfo` - Information needed for optimizations
#[inline(always)]
fn expr_binary(op: &str,
               left: &Expression,
               right: &Expression,
               base: u8,
               func: &mut HashMap<String, u32>,
               vars: &HashMap<String, (Type, Register)>,
               module: &mut Module,
               oinfo: &OptimizationInfo) {
    let reg_left = base + 1;
    generate_expression(left, reg_left, func, vars, module, oinfo);
    let reg_right = base + 2;
    generate_expression(right, reg_right, func, vars, module, oinfo);

    let mut instruction = Instruction {
        opcode: ops::HLT,
        target: base,
        left: base + 1,
        right: base + 2
    };

    match op.as_ref() {
        "+" => instruction.opcode = ops::ADD,
        "-" => instruction.opcode = ops::SUB,
        "*" => instruction.opcode = ops::MUL,
        "/" => instruction.opcode = ops::DIV,
        "&" => instruction.opcode = ops::AND,
        "|" => instruction.opcode = ops::OR,
        "==" => instruction.opcode = ops::EQ,
        "<" => instruction.opcode = ops::LT,
        "<=" => instruction.opcode = ops::LE,
        ">" => instruction.opcode = ops::GT,
        ">=" => instruction.opcode = ops::GE,
        "!=" => instruction.opcode = ops::NEQ,
        _ => panic!("Invalid operation")
    }

    module.code.push(instruction);
}

/// Generate instructions for an unary operation.
///
/// # Arguments
///
/// * `op` - Name of the unary operation
/// * `left` - Only operand of the operation
/// * `base` - Base register of the expression, return value is stored here
/// * `func` - Lookup table for function table entries
/// * `vars` - A variable assignment for all child expressions
/// * `module` - Module to be filled with constant/function/code storage
/// * `oinfo` - Information needed for optimizations
#[inline(always)]
fn expr_unary(op: &str,
              left: &Expression,
              base: u8,
              func: &mut HashMap<String, u32>,
              vars: &HashMap<String, (Type, Register)>,
              module: &mut Module,
              oinfo: &OptimizationInfo) {
    let reg_left = base + 1;
    generate_expression(left, reg_left, func, vars, module, oinfo);

    let mut instruction = Instruction {
        opcode: ops::HLT,
        target: base,
        left: base + 1,
        right: 0
    };

    match op.as_ref() {
        "~" => instruction.opcode = ops::NOT,
        "write" => instruction.opcode = ops::WRI,
        _ => panic!("Invalid operation")
    }

    module.code.push(instruction);
}

/// Generate instructions for a nullary operation.
///
/// # Arguments
///
/// * `op` - Name of the nullary operation
/// * `base` - Base register of the expression, return value is stored here
/// * `module` - Module to be filled with constant/function/code storage
#[inline(always)]
fn expr_nullary(op: &str,
                base: u8,
                module: &mut Module) {
    let mut instruction = Instruction {
        opcode: ops::HLT,
        target: base,
        left: 0,
        right: 0
    };

    match op.as_ref() {
        "read" => instruction.opcode = ops::RDI,
        _ => panic!("Invalid operation")
    }

    module.code.push(instruction);

}

/// Generate instructions for a function call.
///
/// # Arguments
///
/// * `name` - Name of the function
/// * `param` - List of parameters, expressions
/// * `base` - Base register of the expression, return value is stored here
/// * `func` - Lookup table for function table entries
/// * `vars` - A variable assignment for all child expressions
/// * `module` - Module to be filled with constant/function/code storage
/// * `oinfo` - Information needed for optimization
#[inline(always)]
fn expr_call(name: &str,
             param: &[Expression],
             base: u8,
             func: &mut HashMap<String, u32>,
             vars: &HashMap<String, (Type, Register)>,
             module: &mut Module,
             oinfo: &OptimizationInfo) {
    let index = {
        match func.get(name) {
            Some(index) => *index,
            _ => panic!("Function {} is not defined", name)
        }
    };

    // Process each parameter expression before making the actual call
    let mut tmp_base = base;
    let mut tmp_param = reg::VAL;
    let mut tmp_instructions: Vec<Instruction> = Vec::new();
    let mut mov_instruction = if oinfo.tail {
        Instruction {
            opcode: ops::MOV,
            target: tmp_param,
            left: tmp_base,
            right: 0
        }
    } else {
        Instruction {
            opcode: ops::MVO,
            target: tmp_param,
            left: tmp_base,
            right: 0xFF
        }
    };
    let param_oinfo = OptimizationInfo {
        func_name: oinfo.func_name,
        tail: false
    };

    for p in param {
        tmp_base += 1;
        tmp_param += 1;
        generate_expression(p, tmp_base, func, vars, module, &param_oinfo);

        // Pass results to callee parameter registers
        mov_instruction.target = tmp_param;
        mov_instruction.left = tmp_base;
        tmp_instructions.push(mov_instruction.clone());
    }

    // Load results of parameter evaluation and make the call
    module.code.extend(tmp_instructions);
    if oinfo.tail {
        module.code.push(Instruction {
            opcode: ops::JMP,
            target: index as u8,
            left: (index >> 8) as u8,
            right: (index >> 16) as u8
        });
    } else {
        module.code.push(Instruction {
            opcode: ops::CAL,
            target: index as u8,
            left: (index >> 8) as u8,
            right: (index >> 16) as u8
        });
        module.code.push(Instruction {
            opcode: ops::LDR,
            target: base,
            left: 0,
            right: 0
        });
    }
}

/// Generate instructions for a function definition.
///
/// # Arguments
///
/// * `name` - Name of the function
/// * `param` - List of parameter names
/// * `body` - Function body, main expression
/// * `base` - Base register of the expression, return value is stored here
/// * `func` - Lookup table for function table entries
/// * `vars` - A variable assignment for all child expressions
/// * `module` - Module to be filled with constant/function/code storage
/// * `oinfo` - Information needed for optimization
#[inline(always)]
fn expr_fundef(name: &str,
               param: &[String],
               body: &[Expression],
               base: u8,
               func: &mut HashMap<String, u32>,
               vars: &HashMap<String, (Type, Register)>,
               module: &mut Module,
               oinfo: &OptimizationInfo) {
    let index = func.len() as u32;
    let address = module.code.len() as u64;
    func.insert(name.to_string(), index);
    module.functions.push(address);

    let mut base = base;
    let mut vars = vars.clone();
    for p in param {
        vars.insert(p.to_string(), (types::INT, base));
        base += 1;
    }

    let base = base;
    let vars = &vars;
    for expr in body {
        generate_expression(expr, base, func, vars, module, oinfo);
    }

    module.code.push(Instruction {
        opcode: ops::MOV,
        target: reg::VAL,
        left: base,
        right: 0
    });
    module.code.push(Instruction {
        opcode: ops::RET,
        target: 0,
        left: 0,
        right: 0
    });
}

/// Generate instructions for a variable assignment, corresponding to the
/// **let** expression.
///
/// # Arguments
///
/// * `assignment` - A list of tuples, including the variable name and an expression
/// * `body` - The body of a variable assignment is a list of expressions
/// * `base` - Base register of the expression, return value is stored here
/// * `func` - Lookup table for function table entries
/// * `vars` - A variable assignment for all child expressions
/// * `module` - Module to be filled with constant/function/code storage
/// * `oinfo` - Information needed for optimization
///
/// # Remarks
///
/// Variables are evaluated in order of definition. Subsequent variables can access
/// variables previously defined in the same statement.
#[inline(always)]
fn expr_varass(assignment: &[(String, Expression)],
               body: &[Expression],
               base: u8,
               func: &mut HashMap<String, u32>,
               vars: &HashMap<String, (Type, Register)>,
               module: &mut Module,
               oinfo: &OptimizationInfo) {
    let mut tmp_base = base;
    let mut vars = vars.clone();
    for &(ref var, ref expr) in assignment {
        tmp_base += 1;
        generate_expression(expr, tmp_base, func, &vars, module, oinfo);
        vars.insert(var.to_string(), (types::INT, tmp_base));
    }

    let tmp_base = tmp_base;
    let vars = &vars;
    for expr in body {
        generate_expression(expr, tmp_base, func, vars, module, oinfo);
    }

    module.code.push(Instruction {
        opcode: ops::MOV,
        target: base,
        left: tmp_base,
        right: 0
    });
}

/// Generate instructions for a variable use.
///
/// # Arguments
///
/// * `name` - Name of the variable to be loaded
/// * `base` - Base register of the expression, return value is stored here
/// * `vars` - A variable assignment for all child expressions
/// * `module` - Module to be filled with constant/function/code storage
#[inline(always)]
fn expr_variable(name: &str,
                 base: u8,
                 vars: &HashMap<String, (Type, Register)>,
                 module: &mut Module) {
    let (_, reg) = match vars.get(name) {
        Some(index) => *index,
        _ => panic!("Variable {} is not defined", name)
    };

    module.code.push(Instruction {
        opcode: ops::MOV,
        target: base,
        left: reg,
        right: 0
    });
}

/// Generate instructions for a branching operation
///
/// # Arguments
///
/// * `cond` - The condition deciding which branch to take
/// * `yes` - The expressions being executed when the condition is true
/// * `no` - The expressions being executed when the condition is false
/// * `base` - Base register of the expression, return value is stored here
/// * `func` - Lookup table for function table entries
/// * `vaprs` - A variable assignment for all child expressions
/// * `module` - Module to be filled with constant/function/code storage
/// * `oinfo` - Information needed for optimization
#[inline(always)]
fn expr_conditional(cond: &Expression,
                    yes: &[Expression],
                    no: &[Expression],
                    base: u8,
                    func: &mut HashMap<String, u32>,
                    vars: &HashMap<String, (Type, Register)>,
                    module: &mut Module,
                    oinfo: &OptimizationInfo) {
    let condition_opti = OptimizationInfo {
        func_name: oinfo.func_name,
        tail: false
    };
    generate_expression(cond, base, func, vars, module, &condition_opti);

    let jmp_index = module.code.len();
    module.code.push(Instruction {
        opcode: ops::JTF,
        target: base,
        left: 0,
        right: 0
    });

    // Generate every expression except tail
    for expr in &no[..no.len()] {
        generate_expression(expr, base, func, vars, module, &condition_opti);
    }

    // Generate tail expression
    generate_expression(&no[no.len() - 1], base, func, vars, module, oinfo);

    let offset = module.code.len() - jmp_index + 1;
    {
        let jmp = &mut module.code[jmp_index];
        jmp.left = offset as u8;
        jmp.right = (offset >> 8) as u8;
    }

    let jmp_index = module.code.len();
    module.code.push(Instruction {
        opcode: ops::JMF,
        target: 0,
        left: 0,
        right: 0
    });

    // Generate every expression except tail
    for expr in &yes[..yes.len()] {
        generate_expression(expr, base, func, vars, module, &condition_opti);
    }

    // Generate tail expression
    generate_expression(&yes[yes.len() - 1], base, func, vars, module, oinfo);

    let offset = module.code.len() - jmp_index;
    {
        let jmp = &mut module.code[jmp_index];
        jmp.target = offset as u8;
        jmp.left = (offset >> 8) as u8;
        jmp.right = (offset >> 16) as u8;
    }
}
