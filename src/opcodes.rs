pub mod ill {
    use interpreter::ill::{ReadHead, Register, Instruction, IllError};
    use opcodes::ill::ExpressionType::*;
    use std::default::Default;
    use std::ascii::AsciiExt;

    #[derive(Debug, Clone)]
    pub enum ExpressionType {
        IntegerLiteral(usize),
        StringLiteral(String),
        ContainerReference(String),
        // both stacks and variables, no difference (Will search current instruction before searching registers)
        RegisterReference(String),
        // Stack Name
        VariableReference(String),
        InstructionReference(String, Vec<String>),
    }


    impl ExpressionType {
        pub fn name(&self) -> String {
            String::from(match *self {
                IntegerLiteral(_) => "Integer Literal",
                ContainerReference(_) => "Container Reference",
                RegisterReference(_) => "Register Reference",
                VariableReference(_) => "Variable Reference",
                StringLiteral(_) => "String Literal",
                InstructionReference(_, _) => "Instruction Reference",
            })
        }
    }

    pub fn literal() -> ExpressionType {
        ExpressionType::IntegerLiteral(0 as usize)
    }

    pub fn container() -> ExpressionType {
        ExpressionType::ContainerReference(String::new())
    }

    pub fn register() -> ExpressionType {
        ExpressionType::RegisterReference(String::new())
    }

    pub fn variable() -> ExpressionType {
        ExpressionType::VariableReference(String::new())
    }

    pub fn s_literal() -> ExpressionType {
        ExpressionType::StringLiteral(String::new())
    }

    pub fn inst_ref() -> ExpressionType { ExpressionType::InstructionReference(String::new(), Vec::new()) }

    pub fn r_literal(it: usize) -> ExpressionType { ExpressionType::IntegerLiteral(it) }

    pub fn r_container(it: String) -> ExpressionType { ExpressionType::ContainerReference(it) }

    pub fn r_register(it: String) -> ExpressionType { ExpressionType::RegisterReference(it) }

    pub fn r_variable(it: String) -> ExpressionType { ExpressionType::VariableReference(it) }

    pub fn r_string(it: String) -> ExpressionType { ExpressionType::StringLiteral(it) }

    // i've always wanted a modular language...
    pub fn default_opcodes() -> Vec<OpCode> {
        let mut opcodes: Vec<OpCode> = Vec::new();
        opcodes.push(OpCode::new("mov").expecting(literal()).expecting(container()));
        opcodes.push(OpCode::new("mvv").expecting(container()).expecting(variable()));
        opcodes.push(OpCode::new("add").expecting(literal()).expecting(variable()));
        opcodes.push(OpCode::new("mak").expecting(s_literal()).expecting(literal()));
        opcodes.push(OpCode::new("dis").expecting(container()));
        opcodes.push(OpCode::new("do").expecting(inst_ref()));
        opcodes.push(OpCode::new("del").expecting(variable()));
        opcodes.push(OpCode::new("pt").expecting(s_literal()));
        opcodes.push(OpCode::new("ptl").expecting(s_literal()));
        opcodes.push(OpCode::new("if").expecting(inst_ref()).expecting(inst_ref()).expecting(inst_ref()));
        opcodes
    }

    #[derive(Default, Debug, Clone)]
    pub struct OpCode {
        pub name: String,
        pub arguments: Vec<ExpressionType>,
        pub location: Option<ReadHead>,
    }


    impl OpCode {
        pub fn new_str(name: String) -> OpCode {
            OpCode {
                name,
                ..Default::default()
            }
        }

        pub fn new(name: &'static str) -> OpCode {
            OpCode::new_str(String::from(name))
        }


        // also named 'with'
        pub fn expecting(self, some: ExpressionType) -> OpCode {
            let mut args = self.arguments;
            args.push(some);
            OpCode {
                arguments: args,
                ..self
            }
        }

        fn instruction_exists(&self, name: &String, insts: Vec<Instruction>) -> bool {
            insts.iter().find(|x| x.name == *name).is_some()
        }
        fn register_exists(&self, name: String, global: bool, registers: Option<&Vec<Register>>, scope: Option<&mut Vec<Register>>) -> bool {
            if global {
                return registers.unwrap().iter().find(|x| x.identifier == *name).is_some();
            } else {
                return scope.unwrap().iter().find(|x| x.identifier == *name).is_some();
            }
        }
        fn g_register_exists(&self, name: String, g_registers: &Vec<Register>) -> bool { self.register_exists(name, true, Some(g_registers), None) }
        fn l_register_exists(&self, name: String, scope: &mut Vec<Register>) -> bool { self.register_exists(name, false, None, Some(scope)) }


        pub fn execute(&self, debug: bool, registers: &mut Vec<Register>, mut o_insts: Vec<Instruction>, scope: &mut Vec<Register>) -> Result<(), IllError> {
            let rh_err: ReadHead = self.location.unwrap().clone();
            fn get_and_execute(name: &String, debug: bool, registers: &mut Vec<Register>, mut insts: Vec<Instruction>, scope: &mut Vec<Register>) -> Result<usize, IllError> {
                let clone = insts.clone();
                insts.iter_mut().find(|x| x.name == *name).unwrap().c_execute(debug, registers, clone, scope)
            }
            match &*self.name.to_lowercase() {
                "mak" => {
                    if let ExpressionType::StringLiteral(ref identifier) = self.arguments[0] {
                        if self.g_register_exists(identifier.clone(), registers) {
                            return Err(IllError::RegisterRedefinition(rh_err, identifier.clone(), Some(register().name())));
                        } else if self.l_register_exists(identifier.clone(), scope) {
                            return Err(IllError::RegisterRedefinition(rh_err, identifier.clone(), Some(variable().name())));
                        }
                        if let ExpressionType::IntegerLiteral(value) = self.arguments[1] {
                            if identifier.eq_ignore_ascii_case("res") {
                                return Err(IllError::RegisterRedefinition(rh_err, identifier.clone(), Some(format!("default register {:?}", identifier))));
                            }
                            scope.push(Register { identifier: identifier.clone(), value, is_variable: true });
                            if debug {
                                println!("Added variable {} => {}", identifier, value);
                            }
                        }
                    }
                }
                "mvv" => {
                    if let ExpressionType::ContainerReference(ref value) = self.arguments[0] {
                        if let ExpressionType::VariableReference(ref identifier) = self.arguments[1] {
                            let cont = if !self.g_register_exists(value.clone(), registers) {
                                if !self.l_register_exists(value.clone(), scope) {
                                    return Err(IllError::NonExistentRegister(rh_err, value.clone()));
                                } else {
                                    scope.iter_mut().find(|x| x.identifier == *value).unwrap().value
                                }
                            } else {
                                registers.iter_mut().find(|x| x.identifier == *value).unwrap().value
                            };
                            let reg = scope.iter_mut().find(|x| x.identifier == *identifier).unwrap();
                            println!("Added {} onto {}", cont, reg.identifier);
                            reg.value += cont;
                        }
                    }
                }
                "add" => {
                    if let ExpressionType::IntegerLiteral(ref value) = self.arguments[0] {
                        if let ExpressionType::VariableReference(ref variable) = self.arguments[1] {
                            if !self.l_register_exists(variable.clone(), scope) {
                                return Err(IllError::NonExistentRegister(rh_err, variable.clone()));
                            } else {
                                let reg = scope.iter_mut().find(|x| x.identifier == *variable).unwrap();
                                reg.value += *value as usize;
                            }
                        }
                    }
                }
                "mov" => {
                    if let ExpressionType::IntegerLiteral(ref value) = self.arguments[0] {
                        if let ExpressionType::ContainerReference(ref identifier) = self.arguments[1] {
                            if !self.g_register_exists(identifier.clone(), registers) {
                                if !self.l_register_exists(identifier.clone(), scope) {
                                    return Err(IllError::NonExistentRegister(rh_err, identifier.clone())); // Error is implemented but will never be thrown because the it wont compile if the register doesnt exist
                                } else {
                                    let reg = scope.iter_mut().find(|x| x.identifier == *identifier).unwrap();
                                    reg.value = *value as usize;
                                }
                            } else {
                                if debug {
                                    println!("Moved {} onto {}.", value, identifier);
                                }
                                let reg = registers.iter_mut().find(|x| x.identifier == *identifier).unwrap();
                                reg.value = *value as usize;
                            }
                        }
                    }
                }
                "dis" => {
                    if let ExpressionType::ContainerReference(ref identifier) = self.arguments[0] {
                        let mut value: usize = 0;
                        if !self.g_register_exists(identifier.clone(), registers) {
                            if !self.l_register_exists(identifier.clone(), scope) {
                                return Err(IllError::NonExistentRegister(rh_err, identifier.clone()));
                            } else {
                                value = scope.iter().find(|x| x.identifier == *identifier).unwrap().value;
                            }
                        } else {
                            value = registers.iter().find(|x| x.identifier == *identifier).unwrap().value;
                        }
                        println!("{} = {}", identifier, value);
                    }
                }
                "del" => {
                    if let ExpressionType::VariableReference(ref name) = self.arguments[0] {
                        // remove second lookup...
                        let clone = scope.clone();
                        let reg = clone.iter().find(|x| x.identifier == *name);
                        if reg.is_some() {
                            let x_name = reg.unwrap().identifier.clone();
                            let pos = scope.iter().position(|x| x.identifier == x_name).unwrap();
                            scope.remove(pos);
                        } else {
                            return Err(IllError::NonExistentRegister(rh_err, name.clone()));
                        }
                    }
                }
                "pt" => {
                    if let ExpressionType::StringLiteral(ref s) = self.arguments[0] {
                        print!("{}", s);
                    }
                }
                "ptl" => {
                    if let ExpressionType::StringLiteral(ref s) = self.arguments[0] {
                        println!("{}", s);
                    }
                }
                "if" => {
                    if let ExpressionType::InstructionReference(ref inst, ref captures) = self.arguments[0] {
                        if let ExpressionType::InstructionReference(ref a_inst, ref a_captures) = self.arguments[1] {
                            if let ExpressionType::InstructionReference(ref b_inst, ref b_captures) = self.arguments[2] {
                                let nested_clone = o_insts.clone();
                                if !self.instruction_exists(inst, nested_clone) {
                                    return Err(IllError::NonExistentInstruction(rh_err, inst.clone()));
                                }
                                let result = get_and_execute(inst, debug, registers, o_insts.clone(), scope);
                                if result.is_err() {
                                    return Err(result.err().unwrap());
                                } else {
                                    if !self.instruction_exists(a_inst, o_insts.clone()) {
                                        return Err(IllError::NonExistentInstruction(rh_err, a_inst.clone()));
                                    } else if !self.instruction_exists(b_inst, o_insts.clone()) {
                                        return Err(IllError::NonExistentInstruction(rh_err, b_inst.clone()));
                                    }
                                    if result.unwrap() == 0 {
                                        o_insts.clone().iter_mut().find(|x| x.name == *b_inst).unwrap().c_execute(debug, registers, o_insts.clone(), scope);
                                    } else {
                                        o_insts.clone().iter_mut().find(|x| x.name == *a_inst).unwrap().c_execute(debug, registers, o_insts.clone(), scope);
                                    }
                                }
                            }
                        }
                    }
                }
                "do" => {
                    if let ExpressionType::InstructionReference(ref inst, ref captures) = self.arguments[0] {
                        if self.instruction_exists(inst, o_insts.clone()) {
                            let copy = o_insts.clone();
                            o_insts.iter_mut().find(|x| x.name == *inst).unwrap().c_execute(debug, registers, copy, scope).unwrap();
                        } else {
                            return Err(IllError::NonExistentInstruction(rh_err, inst.clone()));
                        }
                    }
                }
                _ => ()
            }
            Ok(())
        }
    }
}