pub mod ill {
    use std::fs::File;
    use std::io::Read;
    use std::iter::Peekable;
    use std::str::Chars;
    use std::error::Error;
    use std::fmt;
    use std::fmt::{Display, Formatter};

    use opcodes::ill::OpCode;
    use opcodes::ill::ExpressionType;
    use opcodes::ill::{s_literal, r_container, r_literal, r_register, r_variable};

    use NamedFile;
    use IllError::*;

    const NEWLINE: char = '\n';
    const Register_DEF: char = '+';
    const DEF_END: char = ';';


    // instructions
    const INST_DEF: char = '$';

    const INST_PARAM_BEGIN: char = '(';
    const INST_PARAM_END: char = ')';

    const INST_CODES_BEGIN: char = '{';
    const INST_CODES_END: char = '}';

    // commands
    const COMMENT_SINGLE_LINE: char = '|';

    #[derive(Default, Debug)]
    pub struct Register {
        pub identifier: String,
        pub value: usize,
        pub is_variable: bool,    }

    #[derive(Debug)]
    pub struct EnhancedFile {
        file: File,
        filename: String,
        content: String,
    }

    impl Clone for EnhancedFile {
        fn clone(&self) -> EnhancedFile {
            EnhancedFile {
                filename: self.filename.clone(),
                content: self.content.clone(),
                file: self.file.try_clone().expect("Faield to copy file..."),
            }
        }
    }

    #[derive(Default, Debug, Clone)]
    pub struct ReadHead {
        column: i32,
        line: i32,
    }

    #[derive(Debug)]
    pub enum IllError {
        RegisterRefinition(ReadHead, String),
        NoRegistersFound(EnhancedFile),
        UnexpectedCharacter(ReadHead, char, Option<String>),
        InstructionRedefinition(ReadHead, String),
        UnknownOpCode(ReadHead, String),
        InvalidOpCodeArguments(ReadHead, String),
        OpCodeArgumentMismatch(ReadHead, String, i32, i32),
        NoMainInstruction(),
        OpCodeInvalidArgument(ReadHead, ExpressionType, String), // wanted, got
        OpCodeInvalidContainerRefrence(ReadHead, ExpressionType, String, String)
    }

    impl Error for IllError {
        fn description(&self) -> &str {
            match *self {
                RegisterRefinition(_, _) => "A Register redefinition was attempted.",
                NoRegistersFound(_) => "No Register definitions found.",
                UnexpectedCharacter(_, _, _) => "Unexpected character found.",
                InstructionRedefinition(_, _) => "A instruction redefinition was attempted.",
                UnknownOpCode(_, _) => "An unknown OpCode was used.",
                InvalidOpCodeArguments(_, _) => "An invalid instruction for an OpCode was found.",
                OpCodeArgumentMismatch(_, _, _, _) => "Opcode has too few or many arguments.",
                NoMainInstruction() => "No Main Instruction was found",
                OpCodeInvalidArgument(_, _, _) => "Argument mismatch in OpCode"
                OpCodeInvalidContainerRefrence(_, _, _, _) => "Container mismatch in OpCode"
            }
        }
    }

    impl IllError {
        pub fn name(&self) -> String {
            String::from(match *self {
                RegisterRefinition(_, _) => "Register Redefinition",
                NoRegistersFound(_) => "No Register Found",
                UnexpectedCharacter(_, _, _) => "Unexpected Character",
                InstructionRedefinition(_, _) => "Instruction Redefinition",
                UnknownOpCode(_, _) => "Unknown OpCode",
                InvalidOpCodeArguments(_, _) => "Invalid OpCode Instruction",
                OpCodeArgumentMismatch(_, _, _, _) => "OpCode Argument Length Mismatch",
                NoMainInstruction() => "No Main Instruction",
                OpCodeInvalidArgument(_, _, _) => "Argument Mismatch"
                OpCodeInvalidContainerRefrence(_, _, _, _) => "Container Mismatch"
            })
        }
    }

    impl Display for IllError {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            fn fmt_rh(rh: &ReadHead) -> String {
                format!("[{}:{}]", rh.line, rh.column)
            }
            match *self {
                RegisterRefinition(ref rh, ref name) => {
                    write!(
                        f,
                        "Err@{} => The Register named \"{}\" already exists!",
                        fmt_rh(rh),
                        name
                    )
                }
                InstructionRedefinition(ref rh, ref name) => {
                    write!(
                        f,
                        "Err@{} => The instruction named \"{}\" already exists!",
                        fmt_rh(rh),
                        name
                    )
                }
                NoRegistersFound(ref e_file) => {
                    write!(
                        f,
                        "Cannot find a Register definition for {:?}.",
                        e_file.filename
                    )
                }
                UnexpectedCharacter(ref rh, ch, ref exp) => {
                    write!(
                        f,
                        "Err@{} => Found unexpected character {}{}",
                        fmt_rh(&rh),
                        ch,
                        exp.as_ref().unwrap_or(&String::from("."))
                    )
                }
                UnknownOpCode(ref rh, ref code) => {
                    write!(
                        f,
                        "Err@{} => \"{}\" is not a valid OpCode",
                        fmt_rh(rh),
                        code
                    )
                }
                InvalidOpCodeArguments(ref rh, ref code) => {
                    write!(
                        f,
                        "Err@{} => \"{}\" is not a valid OpCode",
                        fmt_rh(rh),
                        code
                    )
                }
                OpCodeArgumentMismatch(ref rh, ref code, ref exp, ref given) => {
                    write!(
                        f,
                        "Err@{} => \"{}\", invalid amount of arguments, expected {}, but received {}.",
                        fmt_rh(rh),
                        code,
                        exp,
                        given
                    )
                }
                NoMainInstruction() => {
                    write!(f, "No Main instruction was found for any input files.")
                }
                OpCodeInvalidArgument(ref rh, ref e_type, ref got) => {
                    write!(f, "Err@{} => Expected a {}, but got \"{}\" instead.", fmt_rh(rh), 
                    e_type.name(), got)
                }
                OpCodeInvalidContainerRefrence(ref rh, ref e_type, ref got, ref msg) => {
                    write!(f, "Err@{} => Expected a {}, but got \"{}\" instead: {}.", fmt_rh(rh), e_type.name(), got, msg)
                }
            }
        }
    }

    impl ReadHead {
        fn new() -> ReadHead {
            ReadHead { line: 1, column: 1 }
        }

        fn new_by(&self, line: i32, col: i32) -> ReadHead {
            ReadHead {
                line: self.line + line,
                column: self.column + col,
            }
        }

        fn advance_by(&mut self, line: i32, col: i32) {
            self.column += col;
            self.line += line;
        }

        fn advance(&mut self, ch: char) {
            if ch == NEWLINE {
                self.advance_by(1, 0);
                self.column = 0;
            } else {
                self.advance_by(0, 1);
            }
        }
    }

    #[derive(Default, Debug)]
    struct Instruction {
        name: String,
        codes: Vec<OpCode>,
        scope: Vec<Register>,
        arguments: Vec<String>,
        is_main: bool,
    }

    impl Instruction {
        fn find_scoped_register(&self, name: String) -> Option<&Register> {
            self.scope.iter().find(|&x| x.identifier == name)
        }
        fn does_scoped_register_exist(&self, name: String) -> bool {
            self.find_scoped_register(name).is_some() 
        }
    }

    #[derive(Default)]
    pub struct Interpreter {
        debug: bool,
        files: Vec<EnhancedFile>,
        opcodes: Vec<OpCode>, // valid opcodes
        registers: Vec<Register>,
        instructions: Vec<Instruction>,
    }

    #[derive(Default)]
    struct InstSwitchBox {
        is_reading_definition: bool,
        is_reading_arguments: bool,
        is_reading_codes: bool,
    }

    fn dump_until(head: &mut ReadHead, it: &mut Peekable<Chars>, ch: Vec<char>) {
        let _ = traverse_read(head, read_until_spare_ws(it, ch));
    }

    fn read_until_spare_ws(it: &mut Peekable<Chars>, ch: Vec<char>) -> (i32, i32, String) {
        let z = it.take_while(|c| !ch.contains(c)).collect::<String>();
        let nl = newlines(&z);
        (nl, z.len() as i32 - nl, z.chars().collect::<String>())
    }

    fn newlines(x: &String) -> i32 {
        x.chars().filter(|x| *x == NEWLINE).count() as i32
    }

    fn read_until(it: &mut Peekable<Chars>, ch: Vec<char>) -> (i32, i32, String) {
        let z = it.take_while(|c| !ch.contains(c)).collect::<String>();
        let nl = newlines(&z);
        (
            nl,
            z.len() as i32 - nl,
            z.chars().filter(|c| !c.is_whitespace()).collect::<String>(),
        )
    }

    fn any_exists_until(it: &mut Peekable<Chars>, exists: Vec<char>, until: Vec<char>) -> bool {
        let (_, _, data) = read_until(it, until);
        data.chars().find(|x| exists.contains(x)).is_some()
    }

    fn traverse_read(head: &mut ReadHead, data: (i32, i32, String)) -> String {
        let (row, col, dat) = data;
        head.advance_by(row, col);
        dat
    }


    impl Interpreter {
        fn find_opcode(&self, name: String) -> Option<&OpCode> {
            self.opcodes.iter().find(|x: &&OpCode| x.name == name)
        }
        
        fn does_opcode_exist(&self, name: String) -> bool {
            self.find_opcode(name).is_some()
        }

        pub fn new(debug: bool, sources: Vec<NamedFile>, opcodes: Vec<OpCode>) -> Interpreter {
            if debug {
                println!("Making Interpreter with opcodes {:?}", opcodes);
            }
            Interpreter {
                opcodes: opcodes,
                debug: debug,
                files: sources
                    .iter()
                    .map(|nf| {
                        let mut content = String::new();
                        let mut clone = nf.file.try_clone().expect(&*format!(
                            "[ERROR!]: could not create a copy of: {:?}",
                            nf.name
                        ));
                        let sz = clone.read_to_string(&mut content).unwrap_or(0);
                        if debug {
                            println!("[:] read {} bytes for {:?}", sz, nf.file);
                            println!("content = `{}`", content);
                        }
                        EnhancedFile {
                            filename: nf.name.clone(),
                            file: clone,
                            content: content,
                        }
                    })
                    .collect(),
                ..Default::default()
            }
        }

        fn find_Register(&self, name: String) -> Option<&Register> {
            self.registers.iter().find(|x: &&Register| x.identifier == name)
        }

        fn does_register_exist(&self, name: String) -> bool {
            self.find_Register(name).is_some()
        }

        fn find_instruction(&self, name: String) -> Option<&Instruction> {
            self.instructions.iter().find(
                |x: &&Instruction| x.name == name,
            )
        }

        fn does_instruction_exist(&self, name: String) -> bool {
            self.find_instruction(name).is_some()
        }

        fn parse_code(&self, rh: ReadHead, inst: &Instruction, code: String) -> Result<OpCode, IllError> {
            let data: Vec<String> = code.split(' ').map(String::from).collect::<Vec<String>>();
            let code_name = data[0].clone();
            let nls = newlines(&code) as usize;
            let error_rh = rh.new_by(-(nls as i32), ((-rh.column) + code.len() as i32));
            if !self.does_opcode_exist(code_name.clone()) {
                return Err(IllError::UnknownOpCode(
                    error_rh,
                    data[0].clone(),
                ));
            }
            let opcode = self.find_opcode(code_name.clone()).unwrap().clone();
            if (data.len() - 1) != opcode.arguments.len() {
                return Err(IllError::OpCodeArgumentMismatch(
                    error_rh,
                    data[0].clone(),
                    opcode.arguments.len() as i32,
                    (data.len() - 1) as i32,
                ));
            }

            fn is_arg_literal(arg: String) -> bool {
                arg.parse::<usize>().is_ok()
            }

            fn is_arg_string(arg: String) -> bool {
                arg.chars().find(|x| x.is_numeric()).is_none() // just make sure its [A-z]
            }

            let name = opcode.name.clone();
            let mut exp_args = opcode.arguments.clone();
            let mut act_args: Vec<ExpressionType> = Vec::new();
            for i in 0 .. exp_args.len() {
                let expected = exp_args[i].clone();
                let ref argument = data[i + 1];
                println!("arg = {}, expected = {:?}", argument, expected);
                match expected {
                    ExpressionType::IntegerLiteral(_) => {
                        if !is_arg_literal(argument.clone()) {
                            return Err(OpCodeInvalidArgument(
                                error_rh,
                                r_literal(0),
                                argument.clone()
                            ));
                        } else { // See if I can do this by refrence? (looks to be too complicated, I don't want to have to use refcells)
                            act_args.push(ExpressionType::IntegerLiteral(argument.parse::<usize>().unwrap()));
                        }
                    }

                    ExpressionType::StringLiteral(_) => {
                        if !is_arg_string(argument.clone()) {
                            return Err(OpCodeInvalidArgument(
                                error_rh,
                                s_literal(),
                                argument.clone()
                            ));
                        } else {
                            act_args.push(ExpressionType::StringLiteral(argument.clone()))
                        }
                    }

                    ExpressionType::ContainerRefrence(_) => {
                        if !self.does_register_exist(argument.clone()) && !inst.does_scoped_register_exist(argument.clone()) {
                            return Err(OpCodeInvalidArgument(
                                error_rh,
                                r_container(String::new()),
                                argument.clone()
                            ))
                        } else {
                            act_args.push(ExpressionType::ContainerRefrence(argument.clone()))
                        }
                    }
                    ExpressionType::RegisterRefrence(_) => {
                        if !self.does_register_exist(argument.clone()) {
                            return Err(OpCodeInvalidArgument(
                                error_rh,
                                r_register(String::new()),
                                argument.clone()
                            ))
                        } else {
                            act_args.push(ExpressionType::RegisterRefrence(argument.clone()))
                        }
                    }

                    ExpressionType::VariableRefrence(_, _) => {
                        if !self.does_register_exist(argument.clone()) {
                            return Err(OpCodeInvalidArgument(
                                error_rh,
                                r_variable(String::new(), String::new()),
                                argument.clone()
                            ))
                        } else {
                            act_args.push(ExpressionType::VariableRefrence(inst.name.clone(), argument.clone()))
                        }
                    }
                }
            }

                
            Ok(OpCode {
                name: code_name,
                arguments: act_args,
            })
        }

        fn scan_instructions(&mut self) -> Result<(), IllError> {

            fn read_inst_def(it: &mut Peekable<Chars>) -> (i32, i32, String) {
                read_until(it, vec![INST_PARAM_BEGIN])
            }

            for e_file in &self.files {
                let mut it = e_file.content.chars().peekable();
                let mut head: ReadHead = ReadHead::new();
                let mut cur_inst: Instruction = Default::default();
                let mut cur_inst_sb: InstSwitchBox = Default::default();
                while let Some(x) = it.next() {
                    head.advance(x);
                    if x == COMMENT_SINGLE_LINE {
                        dump_until(&mut head, it.by_ref(), vec![NEWLINE]);
                    } else if x == INST_DEF {
                        if cur_inst_sb.is_reading_definition {
                            return Err(UnexpectedCharacter(
                                head,
                                x,
                                Some(String::from(" expecting instruction identifier.")),
                            ));
                        } else {
                            cur_inst_sb.is_reading_definition = true;
                        }
                        if cur_inst_sb.is_reading_definition {
                            cur_inst.is_main = *it.peek().unwrap() == INST_DEF;
                            let Register_name = traverse_read(&mut head, read_inst_def(it.by_ref()));
                            cur_inst.name = Register_name;
                            cur_inst_sb.is_reading_arguments = true;
                            let params_unsp =
                                traverse_read(
                                    &mut head,
                                    read_until_spare_ws(it.by_ref(), vec![INST_PARAM_END]),
                                );
                            let params: Vec<_> = params_unsp
                                .split(" ")
                                .map(|x: &str| String::from(x))
                                .collect();
                            cur_inst.arguments = params;
                            cur_inst_sb.is_reading_arguments = false;
                            if !any_exists_until(
                                &mut it.clone(),
                                vec![INST_CODES_BEGIN],
                                vec![INST_CODES_END],
                            )
                            {
                                return Err(UnexpectedCharacter(
                                    head,
                                    x,
                                    Some(format!(
                                        " expecting instruction code beginning \"{}\".",
                                        INST_CODES_BEGIN
                                    )),
                                ));
                            }
                            dump_until(&mut head, it.by_ref(), vec![INST_CODES_BEGIN]);
                            while it.peek().is_some() && *it.peek().unwrap() != INST_CODES_END {
                                if !any_exists_until(
                                    &mut it.clone(),
                                    vec![DEF_END],
                                    vec![INST_CODES_END],
                                )
                                {
                                    // break because no codes
                                    break;
                                }
                                if *it.peek().unwrap() == COMMENT_SINGLE_LINE {
                                    dump_until(&mut head, it.by_ref(), vec![NEWLINE]);
                                }
                                let raw_code = traverse_read(
                                    &mut head,
                                    read_until_spare_ws(it.by_ref(), vec![DEF_END]),
                                );
                                let code = String::from(raw_code.trim());
                                let res = self.parse_code(head.clone(), &cur_inst, code.clone());
                                if res.is_err() {
                                    return Err(res.err().unwrap());
                                }
                                cur_inst.codes.push(res.ok().unwrap());
                                println!("found code \"{}\"", code);
                            }
                            cur_inst_sb.is_reading_codes = false;
                            if self.does_instruction_exist(cur_inst.name.clone()) {
                                return Err(IllError::InstructionRedefinition(
                                    head.new_by(0, -(cur_inst.name.len() as i32)),
                                    cur_inst.name,
                                ));
                            }
                            self.instructions.push(cur_inst);
                            cur_inst = Default::default();
                            cur_inst_sb = Default::default();
                        }
                    }
                }
            }
            if self.instructions.len() == 0 {
                return Err(NoMainInstruction());
            } else if self.instructions.len() == 1 {
                self.instructions[0].is_main = true;
            }
            println!("insts = {:?}", self.instructions);
            Ok(())
        }

        fn create_registers(&mut self) -> Result<(), IllError> {

            for e_file in &self.files {
                let mut iter = e_file.content.chars().peekable();
                let mut head: ReadHead = ReadHead::new();
                let mut has_found_registers: bool = false;
                while let Some(x) = iter.next() {
                    head.advance(x);
                    if !x.is_whitespace() {
                        if x == Register_DEF {
                            has_found_registers = true;
                            while iter.peek().is_some() && *iter.peek().unwrap() != NEWLINE {
                                let Register_name = traverse_read(
                                    &mut head,
                                    read_until(iter.by_ref(), vec![DEF_END]),
                                );
                                if self.does_register_exist(Register_name.clone()) {
                                    let err_str = Register_name.clone();
                                    return Err(RegisterRefinition(head, err_str));
                                }
                                self.registers.push(Register {
                                    identifier: Register_name,
                                    is_variable: false,
                                    ..Default::default()
                                });
                                continue;
                            }
                        }
                    }
                }
                if !has_found_registers {
                    return Err(NoRegistersFound(e_file.clone()));
                } else if self.debug {
                    println!("Found registers: {:?}", self.registers);
                }
            }
            Ok(())
        }

        pub fn begin_parsing(&mut self) -> Result<(), IllError> {
            let res = self.create_registers();
            if res.is_err() {
                return res;
            }

            let res = self.scan_instructions();
            if res.is_err() {
                return res;
            }

            Ok(())
        }
    }
}