pub mod ill {
    use interpreter::ill::{ReadHead, Register, Interpreter, Instruction, IllError};
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
        VariableReference(String, String) // Instruction Name, Variable Name
    }


    impl ExpressionType {
        pub fn name(&self) -> String {
            String::from(match *self {
                IntegerLiteral(_) => "Integer Literal",
                ContainerReference(_) => "Container Reference",
                RegisterReference(_) => "Register Reference",
                VariableReference(_, _) => "Variable Reference",
                StringLiteral(_) => "String Literal"
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
        ExpressionType::VariableReference(String::new(), String::new())
    }

    pub fn s_literal() -> ExpressionType {
        ExpressionType::StringLiteral(String::new())
    }

    pub fn r_literal(it: usize) -> ExpressionType { ExpressionType::IntegerLiteral(it) }

    pub fn r_container(it: String) -> ExpressionType { ExpressionType::ContainerReference(it) }

    pub fn r_register(it: String) -> ExpressionType { ExpressionType::RegisterReference(it) }

    pub fn r_variable(inst_name: String, it: String) -> ExpressionType { ExpressionType::VariableReference(inst_name, it) }

    pub fn r_string(it: String) -> ExpressionType { ExpressionType::StringLiteral(it) }

    pub fn do_opcode(code: OpCode) {
        // placeholder
    }

    // i've always wanted a modular language...
    pub fn default_opcodes() -> Vec<OpCode> {
        let mut opcodes: Vec<OpCode> = Vec::new();
        opcodes.push(OpCode::new("mov").expecting(literal()).expecting(container()));
        opcodes.push(OpCode::new("mak").expecting(s_literal()).expecting(literal()));
        opcodes.push(OpCode::new("cop").expecting(container()).expecting(container()));
        opcodes
    } // lol why the FUCCC didnt i use enums kekekekekkekkekekekekek ??
    // i remember now cuz LOL DESTRUCTORS LOL FUNCTIONS LOL OL PLFOL oj ej jknlsnf hehehah fdsfasdklf
    // im in some office building rn and im pseudo-bogging in this code hahahahahahaha what other CRAZY ass developers do this ? ?? hahahahaa
    // lol time to finish writing this stpid code ((i founda BUGGAroo))
    // the fkn thing just sits and stares when it fins an opcode that like, isnt terminated so.....


    // hehe i just fixd that BUG squashed that NOOB lol
    // everyone left the office and its so serene..........


    #[derive(Default, Debug, Clone)]
    pub struct OpCode {
        pub name: String,
        pub arguments: Vec<ExpressionType>,
        pub location: Option<ReadHead>
    }

    impl OpCode {
        pub fn update(&mut self, index: usize, e_type: ExpressionType) {
            self.arguments[index] = e_type;
        }
        pub fn new_str(name: String) -> OpCode {
            OpCode {
                name: name,
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
        pub fn execute(&self, debug: bool, registers: &Vec<Register>, o_insts: Vec<Instruction>, scope: &mut Vec<Register>) -> Result<(), IllError> {
            let rh_err: ReadHead = self.location.unwrap().clone();
            match &*self.name.to_lowercase() {
                "mak" => {
                    if let ExpressionType::StringLiteral(ref identifier) = self.arguments[0] {
                        if registers.iter().find(|x| x.identifier == *identifier).is_some() {
                            return Err(IllError::RegisterRedefinition(rh_err, identifier.clone(), Some(register().name())));
                        } else if scope.iter().find(|x| x.identifier == identifier.clone()).is_some() {
                            return Err(IllError::RegisterRedefinition(rh_err, identifier.clone(), Some(variable().name())));
                        }
                        if let ExpressionType::IntegerLiteral(value) = self.arguments[1] {
                            scope.push(Register { identifier: identifier.clone(), value, is_variable: true });
                            if debug {
                                println!("Added variable {} => {}", identifier, value);
                            }
                        }
                    }
                },
                "mov" => (),
                "cop" => (),
                _ => ()
            }
            Ok(())
        }
    }
}