
pub mod ill {
    use interpreter::ill::Register;
    use opcodes::ill::ExpressionType::*;

    #[derive(Debug, Clone)]
    pub enum ExpressionType {
        IntegerLiteral(usize),
        ContainerRefrence(String), // both stacks and variables, no difference
        RegisterRefrence(String), // Stack Name
        VariableRefrence(String) // Variable Name
    }


    impl ExpressionType {
        pub fn name(&self) -> String {
            String::from(match *self {
                IntegerLiteral(_) => "Integer Literal",
                ContainerRefrence(_) => "Container Refrence",
                RegisterRefrence(_) => "Register Refrence",
                VariableRefrence(_) => "Variable Refrence"
            })
        }
    }

    pub fn literal() -> ExpressionType {
        ExpressionType::IntegerLiteral(0 as usize)
    }

    pub fn container() -> ExpressionType {
        ExpressionType::ContainerRefrence(String::new())
    }

    pub fn register() -> ExpressionType {
        ExpressionType::RegisterRefrence(String::new())
    }

    pub fn variable() -> ExpressionType {
        ExpressionType::VariableRefrence(String::new())
    }

    pub fn r_literal(it: usize) -> ExpressionType { ExpressionType::IntegerLiteral(it) }
    pub fn r_container(it: String) -> ExpressionType { ExpressionType::ContainerRefrence(it) }
    pub fn r_register(it: String) -> ExpressionType { ExpressionType::RegisterRefrence(it) }
    pub fn r_variable(it: String) -> ExpressionType { ExpressionType::VariableRefrence(it) }


    pub fn do_opcode(code: OpCode) {
        // placeholder
    }

    // i've always wanted a modular language...
    pub fn default_opcodes() -> Vec<OpCode> {
        let mut opcodes: Vec<OpCode> = Vec::new();
        opcodes.push(OpCode::new("mov").expecting(container()).expecting(literal()));
        opcodes.push(OpCode::new("mak").expecting(variable()).expecting(literal()));
        opcodes.push(OpCode::new("cop").expecting(container()).expecting(container()));
        opcodes
    }

    #[derive(Default, Debug, Clone)]
    pub struct OpCode {
        pub name: String,
        pub arguments: Vec<ExpressionType>,
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
    }


}