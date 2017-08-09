
pub mod ill {
    use interpreter::ill::Stack;



    // why did i write shit lol
    fn stack_helper(var: bool) -> Stack {
        Stack {
            is_variable: var,
            ..Default::default()
        }
    }

    fn register() -> Stack {
        stack_helper(false)
    }

    fn variable() -> Stack {
        stack_helper(true)
    }


    // i've always wanted a modular language...
    pub fn default_opcodes() -> Vec<OpCode> {
        let mut opcodes: Vec<OpCode> = Vec::new();
        opcodes.push(OpCode::new("mov").expecting(register()).expecting(variable()));
        opcodes
    }

    #[derive(Default, Debug)]
    pub struct OpCode {
        pub name: String,
        pub arguments: Vec<Stack>,
    }

    impl OpCode {
        pub fn new_str(name: String) -> OpCode {
            OpCode {
                name: name,
                ..Default::default()
            }
        }

        pub fn new(name: &'static str) -> OpCode {
            OpCode::new_str(String::from(name))
        }

        pub fn expecting(self, some: Stack) -> OpCode {
            let mut args = self.arguments;
            args.push(some);
            OpCode {
                arguments: args,
                ..self
            }
        }
    }


}