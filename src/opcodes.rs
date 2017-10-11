pub mod ill {
    use interpreter::ill::{ReadHead, Register, Instruction, IllError};
    use opcodes::ill::ExpressionType::*;
    use std::default::Default;

    #[derive(Debug, Clone)]
    pub enum ExpressionType {
        IntegerLiteral(usize),
        StringLiteral(String),
        ContainerReference(String),
        // both stacks and variables, no difference (Will search current instruction before searching registers)
        RegisterReference(String),
        // Stack Name
        VariableReference(String),
        NestedOpCode(usize),
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
                NestedOpCode(_) => "Nested OpCode",
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

    pub fn opcode_result() -> ExpressionType { ExpressionType::NestedOpCode(0 as usize) }

    pub fn inst_ref() -> ExpressionType { ExpressionType::InstructionReference(String::new(), Vec::new()) }

    pub fn r_literal(it: usize) -> ExpressionType { ExpressionType::IntegerLiteral(it) }

    pub fn r_container(it: String) -> ExpressionType { ExpressionType::ContainerReference(it) }

    pub fn r_register(it: String) -> ExpressionType { ExpressionType::RegisterReference(it) }

    pub fn r_variable(it: String) -> ExpressionType { ExpressionType::VariableReference(it) }

    pub fn r_string(it: String) -> ExpressionType { ExpressionType::StringLiteral(it) }

    pub fn r_opcode_res(it: String) -> ExpressionType { ExpressionType::NestedOpCode(0) }

    // i've always wanted a modular language...
    pub fn default_opcodes() -> Vec<OpCode> {
        let mut opcodes: Vec<OpCode> = Vec::new();
        opcodes.push(OpCode::new("mov").expecting(literal()).expecting(container()));
        opcodes.push(OpCode::new("mvv").expecting(container()).expecting(variable()));
        opcodes.push(OpCode::new("mak").expecting(s_literal()).expecting(literal()));
        opcodes.push(OpCode::new("dis").expecting(container()));
        opcodes.push(OpCode::new("do").expecting(inst_ref()));
        opcodes.push(OpCode::new("ret").expecting(opcode_result()));
        opcodes.push(OpCode::new("del").expecting(variable()));
        opcodes.push(OpCode::new("pt").expecting(s_literal()));
        opcodes.push(OpCode::new("ptl").expecting(s_literal()));
        opcodes.push(OpCode::new("if").expecting(opcode_result()).expecting(inst_ref()).expecting(inst_ref()));
        opcodes
    }

    #[derive(Default, Debug, Clone)]
    pub struct OpCode {
        pub name: String,
        pub arguments: Vec<ExpressionType>,
        pub location: Option<ReadHead>,
        pub result: Option<usize>
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
            match &*self.name.to_lowercase() {
                "mak" => {
                    if let ExpressionType::StringLiteral(ref identifier) = self.arguments[0] {
                        if self.g_register_exists(identifier.clone(), registers) {
                            return Err(IllError::RegisterRedefinition(rh_err, identifier.clone(), Some(register().name())));
                        } else if self.l_register_exists(identifier.clone(), scope) {
                            return Err(IllError::RegisterRedefinition(rh_err, identifier.clone(), Some(variable().name())));
                        }
                        if let ExpressionType::IntegerLiteral(value) = self.arguments[1] {
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
                                    return Err(IllError::NonExistentRegister(rh_err, identifier.clone())); // Error is implemented but will never be thrown because the it wont compile if the register doesnt exist
                                } else {
                                    scope.iter_mut().find(|x| x.identifier == *value).unwrap().value
                                }
                            } else {
                                registers.iter_mut().find(|x| x.identifier == *value).unwrap().value
                            };
                            let reg = scope.iter_mut().find(|x| x.identifier == *identifier).unwrap();
                            reg.value += cont;
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
                "do" => {
                    if let ExpressionType::InstructionReference(ref inst, ref captures) = self.arguments[0] {
                        let copy = o_insts.clone();
                        // let c_scope = scope.iter().filter(|x| captures.contains(&x.identifier)).collect();
                        o_insts.iter_mut().find(|x| x.name == *inst).unwrap().c_execute(debug, registers, copy, scope).unwrap();
                    }
                }
                _ => ()
            }
            Ok(())
        }
    }
}