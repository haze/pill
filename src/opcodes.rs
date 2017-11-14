pub mod ill {
    use interpreter::ill::{ReadHead, Register, Instruction, EnhancedFile, AdvancedIllError, IllError};
    use opcodes::ill::ExpressionType::*;
    use std::default::Default;
    use std::ascii::AsciiExt;
    use either::Either;

    const TRUE: f64 = 0f64;
    const FALSE: f64 = 1f64;

    #[derive(Debug, Clone)]
    pub enum ExpressionType {
        IntegerLiteral(f64),
        ProbableLiteral(Either<f64, String>),
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
                ProbableLiteral(_) => "Probable Integer (Variable, or Literal)",
                ContainerReference(_) => "Container Reference",
                RegisterReference(_) => "Register Reference",
                VariableReference(_) => "Variable Reference",
                StringLiteral(_) => "String Literal",
                InstructionReference(_, _) => "Instruction Reference",
            })
        }
    }

    pub fn literal() -> ExpressionType {
        ExpressionType::IntegerLiteral(0 as f64)
    }

    pub fn prob_literal() -> ExpressionType { ExpressionType::ProbableLiteral(Either::Left(FALSE)) }

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


    // i've always wanted a modular language...
    pub fn default_opcodes() -> Vec<OpCode> {
        let mut opcodes: Vec<OpCode> = Vec::new();
        opcodes.push(OpCode::new("mov").expecting(prob_literal()).expecting(container()));
        opcodes.push(OpCode::new("mod").expecting(prob_literal()).expecting(prob_literal()).expecting(s_literal()));
        opcodes.push(OpCode::new("gt").expecting(prob_literal()).expecting(prob_literal()).expecting(s_literal()));
        opcodes.push(OpCode::new("lt").expecting(prob_literal()).expecting(prob_literal()).expecting(s_literal()));
        opcodes.push(OpCode::new("eq").expecting(prob_literal()).expecting(prob_literal()).expecting(s_literal()));
        opcodes.push(OpCode::new("gte").expecting(prob_literal()).expecting(prob_literal()).expecting(s_literal()));
        opcodes.push(OpCode::new("lte").expecting(prob_literal()).expecting(prob_literal()).expecting(s_literal()));
        opcodes.push(OpCode::new("add").expecting(prob_literal()).expecting(container()));
        opcodes.push(OpCode::new("mak").expecting(s_literal()).expecting(prob_literal()));
        opcodes.push(OpCode::new("dis").expecting(container()));
        opcodes.push(OpCode::new("dsl").expecting(container()));
        opcodes.push(OpCode::new("do").expecting(inst_ref()));
        opcodes.push(OpCode::new("dor").expecting(inst_ref()).expecting(s_literal()));
        opcodes.push(OpCode::new("del").expecting(variable()));
        opcodes.push(OpCode::new("pt").expecting(s_literal()));
        opcodes.push(OpCode::new("ptl").expecting(s_literal()));
        opcodes.push(OpCode::new("neg").expecting(container()));
        opcodes.push(OpCode::new("for").expecting(s_literal()).expecting(literal()).expecting(literal()).expecting(literal()).expecting(inst_ref()));
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

        fn get_absolute_value(&self, file: EnhancedFile, rh_err: ReadHead, ei: &Either<f64, String>, registers: &Vec<Register>, scope: &mut Vec<Register>) -> Result<f64, AdvancedIllError> {
            let is_left = ei.is_left();
            let v_clone = ei.clone();
            Ok(if is_left { v_clone.left().unwrap() } else {
                let name = v_clone.right().unwrap();
                if !self.g_register_exists(name.clone(), registers) {
                    if !self.l_register_exists(name.clone(), scope) {
                        let err = IllError::NonExistentRegister(rh_err, name.clone());
                        return Err(AdvancedIllError::new(err, Some(rh_err), file));
                    } else {
                        scope.iter_mut().find(|x| x.identifier == name).unwrap().value
                    }
                } else {
                    registers.iter().find(|x| x.identifier == name).unwrap().value
                }
            })
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


        pub fn execute(&self, file: EnhancedFile, debug: bool, registers: &mut Vec<Register>, mut o_insts: Vec<Instruction>, scope: &mut Vec<Register>) -> Result<(), AdvancedIllError> {
            let rh_err: ReadHead = self.location.unwrap().clone();
            let rh_err_o: Option<ReadHead> = Some(rh_err);
            fn get_and_execute(file: EnhancedFile, name: &String, debug: bool, registers: &mut Vec<Register>, mut insts: Vec<Instruction>, scope: &mut Vec<Register>) -> Result<f64, AdvancedIllError> {
                let clone = insts.clone();
                /* let f_clone = file.try_clone();
                if f_clone.is_some() {
                    Ok(insts.iter_mut().find(|x| x.name == *name).unwrap().c_execute(file.unsafe_clone(), debug, registers, clone, scope));
                }
                Err(f_clone.err()) */
                insts.iter_mut().find(|x| x.name == *name).unwrap().c_execute(file.unsafe_clone(), debug, registers, clone, scope)
            }
            match &*self.name.to_lowercase() {
                "mak" => {
                    if let ExpressionType::StringLiteral(ref identifier) = self.arguments[0] {
                        if self.g_register_exists(identifier.clone(), registers) {
                            let err = IllError::RegisterRedefinition(rh_err, identifier.clone(), Some(register().name()));
                            return Err(AdvancedIllError::new(err, rh_err_o, file));
                        } else if self.l_register_exists(identifier.clone(), scope) {
                            let err = IllError::RegisterRedefinition(rh_err, identifier.clone(), Some(variable().name()));
                            return Err(AdvancedIllError::new(err, rh_err_o, file));
                        }
                        if let ExpressionType::ProbableLiteral(ref value) = self.arguments[1] {
                            if identifier.eq_ignore_ascii_case("res") {
                                let err = IllError::RegisterRedefinition(rh_err, identifier.clone(), Some(format!("default register {:?}", identifier)));
                                return Err(AdvancedIllError::new(err, rh_err_o, file));
                            }
                            let cont = self.get_absolute_value(file.unsafe_clone(), rh_err, value, registers, scope);
                            if cont.is_err() {
                                return Err(cont.err().unwrap());
                            }
                            scope.push(Register { identifier: identifier.clone(), value: cont.ok().unwrap(), is_variable: true });
                            if debug {
                                println!("Added variable {} => {}", identifier, value);
                            }
                        }
                    }
                }
                "neg" => {
                    if let ExpressionType::ContainerReference(ref value) = self.arguments[0] {
                        let reg_ref = if !self.g_register_exists(value.clone(), registers) {
                            if !self.l_register_exists(value.clone(), scope) {
                                let err = IllError::NonExistentRegister(rh_err, value.clone());
                                return Err(AdvancedIllError::new(err, rh_err_o, file));

                            } else {
                                scope.iter_mut().find(|x| x.identifier == *value).unwrap()
                            }
                        } else {
                            registers.iter_mut().find(|x| x.identifier == *value).unwrap()
                        };
                        if reg_ref.value == TRUE {
                            reg_ref.value = FALSE;
                        } else {
                            reg_ref.value = TRUE;
                        }
                    }
                }
                "add" => {
                    if let ExpressionType::ProbableLiteral(ref value) = self.arguments[0] {
                        if let ExpressionType::ContainerReference(ref variable) = self.arguments[1] {
                            let value = self.get_absolute_value(file.unsafe_clone(), rh_err, value, registers, scope);
                            if value.is_err() {
                                return Err(value.err().unwrap());
                            }
                            let res = value.ok().unwrap();
                            if self.l_register_exists(variable.clone(), scope) {
                                let reg = scope.iter_mut().find(|x| x.identifier == *variable).unwrap();
                                reg.value += res;
                            } else if self.g_register_exists(variable.clone(), registers) {
                                let reg = registers.iter_mut().find(|x| x.identifier == *variable).unwrap();
                                reg.value += res;
                            } else {
                                let err = IllError::NonExistentRegister(rh_err, variable.clone());
                                return Err(AdvancedIllError::new(err, rh_err_o, file));
                            }
                        }
                    }
                }
                "mov" => {
                    if let ExpressionType::ProbableLiteral(ref value_x) = self.arguments[0] {
                        if let ExpressionType::ContainerReference(ref identifier) = self.arguments[1] {
                            let value = self.get_absolute_value(file.unsafe_clone(), rh_err, value_x, registers, scope);
                            if value.is_err() {
                                return Err(value.err().unwrap());
                            }
                            let val = value.ok().unwrap();
                            if !self.g_register_exists(identifier.clone(), registers) {
                                if !self.l_register_exists(identifier.clone(), scope) {
                                    let err = IllError::NonExistentRegister(rh_err, identifier.clone()); // Error is implemented but will never be thrown because the it wont compile if the register doesnt exist
                                    return Err(AdvancedIllError::new(err, rh_err_o, file));
                                } else {
                                    let reg = scope.iter_mut().find(|x| x.identifier == *identifier).unwrap();
                                    reg.value = val;
                                }
                            } else {
                                if debug {
                                    println!("Moved {} onto {}.", val, identifier);
                                }
                                let reg = registers.iter_mut().find(|x| x.identifier == *identifier).unwrap();
                                reg.value = val;
                            }
                        }
                    }
                }
                "dsl" => {
                    if let ExpressionType::ContainerReference(ref identifier) = self.arguments[0] {
                        let value;
                        if !self.g_register_exists(identifier.clone(), registers) {
                            if !self.l_register_exists(identifier.clone(), scope) {
                                let err = IllError::NonExistentRegister(rh_err, identifier.clone());
                                return Err(AdvancedIllError::new(err, rh_err_o, file));
                            } else {
                                value = scope.iter().find(|x| x.identifier == *identifier).unwrap().value;
                            }
                        } else {
                            value = registers.iter().find(|x| x.identifier == *identifier).unwrap().value;
                        }
                        println!("{}", value);
                    }
                }
                "dis" => {
                    if let ExpressionType::ContainerReference(ref identifier) = self.arguments[0] {
                        let value;
                        if !self.g_register_exists(identifier.clone(), registers) {
                            if !self.l_register_exists(identifier.clone(), scope) {
                                let err = IllError::NonExistentRegister(rh_err, identifier.clone());
                                let adv_err = Err(AdvancedIllError::new(err, rh_err_o, file));
                                return adv_err;
                            } else {
                                value = scope.iter().find(|x| x.identifier == *identifier).unwrap().value;
                            }
                        } else {
                            value = registers.iter().find(|x| x.identifier == *identifier).unwrap().value;
                        }
                        print!("{}", value);
                    }
                }
                "mod" => {
                    if let ExpressionType::ProbableLiteral(ref t_for) = self.arguments[0] {
                        if let ExpressionType::ProbableLiteral(ref by) = self.arguments[1] {
                            if let ExpressionType::StringLiteral(ref identifier) = self.arguments[2] {
                                let t_for_val = self.get_absolute_value(file.unsafe_clone(), rh_err, t_for, registers, scope);
                                if t_for_val.is_err() {
                                    return Err(t_for_val.err().unwrap());
                                }
                                let by_val = self.get_absolute_value(file.unsafe_clone(), rh_err, by, registers, scope);
                                if by_val.is_err() {
                                    return Err(by_val.err().unwrap());
                                }
                                if self.g_register_exists(identifier.clone(), registers) {
                                    let err = IllError::RegisterRedefinition(rh_err, identifier.clone(), Some(register().name()));
                                    return Err(AdvancedIllError::new(err, rh_err_o, file));
                                } else if self.l_register_exists(identifier.clone(), scope) {
                                    let err = IllError::RegisterRedefinition(rh_err, identifier.clone(), Some(variable().name()));
                                    return Err(AdvancedIllError::new(err, rh_err_o, file));
                                }

                                scope.push(Register {
                                    identifier: identifier.clone(),
                                    value: (t_for_val.ok().unwrap() % by_val.ok().unwrap()) as f64,
                                    is_variable: true
                                })
                            }
                        }
                    }
                }
                "eq" => {
                    if let ExpressionType::ProbableLiteral(ref t_for) = self.arguments[0] {
                        if let ExpressionType::ProbableLiteral(ref by) = self.arguments[1] {
                            if let ExpressionType::StringLiteral(ref identifier) = self.arguments[2] {
                                let t_for_val = self.get_absolute_value(file.unsafe_clone(), rh_err, t_for, registers, scope);
                                if t_for_val.is_err() {
                                    return Err(t_for_val.err().unwrap());
                                }
                                let by_val = self.get_absolute_value(file.unsafe_clone(), rh_err, by, registers, scope);
                                if by_val.is_err() {
                                    return Err(by_val.err().unwrap());
                                }

                                if self.g_register_exists(identifier.clone(), registers) {
                                    let err = IllError::RegisterRedefinition(rh_err, identifier.clone(), Some(register().name()));
                                    return Err(AdvancedIllError::new(err, rh_err_o, file));

                                } else if self.l_register_exists(identifier.clone(), scope) {
                                    let err = IllError::RegisterRedefinition(rh_err, identifier.clone(), Some(variable().name()));
                                    return Err(AdvancedIllError::new(err, rh_err_o, file));

                                }
                                scope.push(Register {
                                    identifier: identifier.clone(),
                                    value: if t_for_val.ok().unwrap() == by_val.ok().unwrap() { TRUE } else { FALSE },
                                    is_variable: true
                                })
                            }
                        }
                    }
                }
                "lt" => {
                    if let ExpressionType::ProbableLiteral(ref t_for) = self.arguments[0] {
                        if let ExpressionType::ProbableLiteral(ref by) = self.arguments[1] {
                            if let ExpressionType::StringLiteral(ref identifier) = self.arguments[2] {
                                let t_for_val = self.get_absolute_value(file.unsafe_clone(), rh_err, t_for, registers, scope);
                                if t_for_val.is_err() {
                                    return Err(t_for_val.err().unwrap());

                                }
                                let by_val = self.get_absolute_value(file.unsafe_clone(), rh_err, by, registers, scope);
                                if by_val.is_err() {
                                    return Err(by_val.err().unwrap());
                                }

                                if self.g_register_exists(identifier.clone(), registers) {
                                    let err = IllError::RegisterRedefinition(rh_err, identifier.clone(), Some(register().name()));
                                    return Err(AdvancedIllError::new(err, rh_err_o, file));

                                } else if self.l_register_exists(identifier.clone(), scope) {
                                    let err = IllError::RegisterRedefinition(rh_err, identifier.clone(), Some(variable().name()));
                                    return Err(AdvancedIllError::new(err, rh_err_o, file));

                                }

                                scope.push(Register {
                                    identifier: identifier.clone(),
                                    value: if t_for_val.ok().unwrap() < by_val.ok().unwrap() { TRUE } else { FALSE },
                                    is_variable: true
                                })
                            }
                        }
                    }
                }
                "gt" => {
                    if let ExpressionType::ProbableLiteral(ref t_for) = self.arguments[0] {
                        if let ExpressionType::ProbableLiteral(ref by) = self.arguments[1] {
                            if let ExpressionType::StringLiteral(ref identifier) = self.arguments[2] {
                                let t_for_val = self.get_absolute_value(file.unsafe_clone(), rh_err, t_for, registers, scope);
                                if t_for_val.is_err() {
                                    return Err(t_for_val.err().unwrap());
                                }
                                let by_val = self.get_absolute_value(file.unsafe_clone(), rh_err, by, registers, scope);
                                if by_val.is_err() {
                                    return Err(by_val.err().unwrap());
                                }

                                if self.g_register_exists(identifier.clone(), registers) {
                                    let err = IllError::RegisterRedefinition(rh_err, identifier.clone(), Some(register().name()));
                                    return Err(AdvancedIllError::new(err, rh_err_o, file));

                                } else if self.l_register_exists(identifier.clone(), scope) {
                                    let err = IllError::RegisterRedefinition(rh_err, identifier.clone(), Some(variable().name()));
                                    return Err(AdvancedIllError::new(err, rh_err_o, file));

                                }

                                scope.push(Register {
                                    identifier: identifier.clone(),
                                    value: if t_for_val.ok().unwrap() > by_val.ok().unwrap() { TRUE } else { FALSE },
                                    is_variable: true
                                })
                            }
                        }
                    }
                }
                "gte" => {
                    if let ExpressionType::ProbableLiteral(ref t_for) = self.arguments[0] {
                        if let ExpressionType::ProbableLiteral(ref by) = self.arguments[1] {
                            if let ExpressionType::StringLiteral(ref identifier) = self.arguments[2] {
                                let t_for_val = self.get_absolute_value(file.unsafe_clone(), rh_err, t_for, registers, scope);
                                if t_for_val.is_err() {
                                    return Err(t_for_val.err().unwrap());
                                }
                                let by_val = self.get_absolute_value(file.unsafe_clone(), rh_err, by, registers, scope);
                                if by_val.is_err() {
                                    return Err(by_val.err().unwrap());
                                }

                                if self.g_register_exists(identifier.clone(), registers) {
                                    let err = IllError::RegisterRedefinition(rh_err, identifier.clone(), Some(register().name()));
                                    return Err(AdvancedIllError::new(err, rh_err_o, file));

                                } else if self.l_register_exists(identifier.clone(), scope) {
                                    let err = IllError::RegisterRedefinition(rh_err, identifier.clone(), Some(variable().name()));
                                    return Err(AdvancedIllError::new(err, rh_err_o, file));

                                }

                                scope.push(Register {
                                    identifier: identifier.clone(),
                                    value: if t_for_val.ok().unwrap() >= by_val.ok().unwrap() { TRUE } else { FALSE },
                                    is_variable: true
                                })
                            }
                        }
                    }
                }
                "lte" => {
                    if let ExpressionType::ProbableLiteral(ref t_for) = self.arguments[0] {
                        if let ExpressionType::ProbableLiteral(ref by) = self.arguments[1] {
                            if let ExpressionType::StringLiteral(ref identifier) = self.arguments[2] {
                                let t_for_val = self.get_absolute_value(file.unsafe_clone(), rh_err, t_for, registers, scope);
                                if t_for_val.is_err() {
                                    return Err(t_for_val.err().unwrap());
                                }
                                let by_val = self.get_absolute_value(file.unsafe_clone(), rh_err, by, registers, scope);
                                if by_val.is_err() {
                                    return Err(by_val.err().unwrap());
                                }

                                if self.g_register_exists(identifier.clone(), registers) {
                                    let err = IllError::RegisterRedefinition(rh_err, identifier.clone(), Some(register().name()));
                                    return Err(AdvancedIllError::new(err, rh_err_o, file));

                                } else if self.l_register_exists(identifier.clone(), scope) {
                                    let err = IllError::RegisterRedefinition(rh_err, identifier.clone(), Some(variable().name()));
                                    return Err(AdvancedIllError::new(err, rh_err_o, file));

                                }

                                scope.push(Register {
                                    identifier: identifier.clone(),
                                    value: if t_for_val.ok().unwrap() <= by_val.ok().unwrap() { TRUE } else { FALSE },
                                    is_variable: true
                                })
                            }
                        }
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
                            let err = IllError::NonExistentRegister(rh_err, name.clone());
                            return Err(AdvancedIllError::new(err, rh_err_o, file));
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
                "for" => {
                    if let ExpressionType::StringLiteral(ref injected_var_name) = self.arguments[0] {
                        if let ExpressionType::IntegerLiteral(ref from) = self.arguments[1] {
                            if let ExpressionType::IntegerLiteral(ref through) = self.arguments[2] {
                                if let ExpressionType::IntegerLiteral(ref step) = self.arguments[3] {
                                    if let ExpressionType::InstructionReference(ref inst, ref captures) = self.arguments[4] {
                                        if self.g_register_exists(injected_var_name.clone(), registers) {
                                            let err = IllError::RegisterRedefinition(rh_err, injected_var_name.clone(), Some(register().name()));
                                            return Err(AdvancedIllError::new(err, rh_err_o, file));
                                        } else if self.l_register_exists(injected_var_name.clone(), scope) {
                                            let err = IllError::RegisterRedefinition(rh_err, injected_var_name.clone(), Some(variable().name()));
                                            return Err(AdvancedIllError::new(err, rh_err_o, file));
                                        }
                                        if !self.instruction_exists(inst, o_insts.clone()) {
                                            let err = IllError::NonExistentInstruction(rh_err, inst.clone());
                                            return Err(AdvancedIllError::new(err, rh_err_o, file));
                                        }
                                        let start = (*from) - 1f64;
                                        let mut clone = o_insts.clone();
                                        scope.push(Register {
                                            identifier: injected_var_name.clone(),
                                            value: start,
                                            is_variable: true,
                                        });
                                        let func = clone.iter_mut().find(|x| x.name == *inst).unwrap();
                                        let mut val = start;
                                        while if val > *through { val > *through } else { val < *through } {
                                            let res = func.c_execute(file.unsafe_clone(), debug, registers, o_insts.clone(), scope);
                                            if res.is_err() {
                                                return Err(res.err().unwrap());
                                            }
                                            val = scope.iter().find(|x| x.identifier == *injected_var_name).unwrap().value;
                                            if from > through {
                                                val -= *step;
                                            } else {
                                                val += *step;
                                            }
                                            scope.iter_mut().find(|x| x.identifier == *injected_var_name).unwrap().value = val;
                                        }
                                    }
                                    let pos = scope.iter().position(|x| x.identifier == *injected_var_name).unwrap();
                                    scope.remove(pos);
                                }
                            }
                        }
                    }
                }
                "if" => {
                    if let ExpressionType::InstructionReference(ref inst, ref captures) = self.arguments[0] {
                        if let ExpressionType::InstructionReference(ref a_inst, ref a_captures) = self.arguments[1] {
                            if let ExpressionType::InstructionReference(ref b_inst, ref b_captures) = self.arguments[2] {
                                let nested_clone = o_insts.clone();
                                if !self.instruction_exists(inst, nested_clone) {
                                    let err = IllError::NonExistentInstruction(rh_err, inst.clone());
                                    return Err(AdvancedIllError::new(err, rh_err_o, file));
                                }
                                let result = get_and_execute(file.unsafe_clone(), inst, debug, registers, o_insts.clone(), scope);
                                if result.is_err() {
                                    return Err(result.err().unwrap());
                                } else {
                                    let unr = result.ok().unwrap();
                                    if !self.instruction_exists(a_inst, o_insts.clone()) {
                                        let err = IllError::NonExistentInstruction(rh_err, a_inst.clone());
                                        return Err(AdvancedIllError::new(err, rh_err_o, file));
                                    } else if !self.instruction_exists(b_inst, o_insts.clone()) {
                                        let err = IllError::NonExistentInstruction(rh_err, b_inst.clone());
                                        return Err(AdvancedIllError::new(err, rh_err_o, file));
                                    }
                                    if unr == TRUE {
                                        let res = o_insts.clone().iter_mut().find(|x| x.name == *a_inst).unwrap().c_execute(file.unsafe_clone(), debug, registers, o_insts.clone(), scope);
                                        if res.is_err() {
                                            return Err(res.err().unwrap());
                                        }
                                    } else {
                                        let res = o_insts.clone().iter_mut().find(|x| x.name == *b_inst).unwrap().c_execute(file.unsafe_clone(), debug, registers, o_insts.clone(), scope);
                                        if res.is_err() {
                                            return Err(res.err().unwrap());
                                        }
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
                            o_insts.iter_mut().find(|x| x.name == *inst).unwrap().c_execute(file.unsafe_clone(), debug, registers, copy, scope).ok().unwrap();
                        } else {
                            let err = IllError::NonExistentInstruction(rh_err, inst.clone());
                            return Err(AdvancedIllError::new(err, rh_err_o, file));
                        }
                    }
                }
                "dor" => {
                    if let ExpressionType::InstructionReference(ref inst, ref captures) = self.arguments[0] {
                        if let ExpressionType::StringLiteral(ref identifier) = self.arguments[1] {
                            if self.g_register_exists(identifier.clone(), registers) {
                                let err = IllError::RegisterRedefinition(rh_err, identifier.clone(), Some(register().name()));
                                return Err(AdvancedIllError::new(err, rh_err_o, file));
                            } else if self.l_register_exists(identifier.clone(), scope) {
                                let err = IllError::RegisterRedefinition(rh_err, identifier.clone(), Some(variable().name()));
                                return Err(AdvancedIllError::new(err, rh_err_o, file));
                            }
                            if self.instruction_exists(inst, o_insts.clone()) {
                                let copy = o_insts.clone();
                                let res = o_insts.iter_mut().find(|x| x.name == *inst).unwrap().c_execute(file.unsafe_clone(), debug, registers, copy, scope);
                                if res.is_ok() {
                                    scope.push(Register {
                                        identifier: identifier.clone(),
                                        value: res.ok().unwrap(),
                                        is_variable: true,
                                    });
                                } else {
                                    return Err(res.err().unwrap());
                                }
                            } else {
                                let err = IllError::NonExistentInstruction(rh_err, inst.clone());
                                return Err(AdvancedIllError::new(err, rh_err_o, file));

                            }
                        }
                    }
                }
                _ => ()
            }
            Ok(())
        }
    }
}