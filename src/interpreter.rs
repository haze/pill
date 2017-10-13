pub mod ill {
    use std::fs::File;
    use std::io::Read;
    use std::iter::Peekable;
    use std::str::Chars;
    use std::error::Error;
    use std::fmt;
    use std::fmt::{Display, Formatter};

    use opcodes::ill::OpCode;
    use opcodes::ill::{ExpressionType};
    use opcodes::ill::{s_literal};

    use pcre::Pcre;

    use NamedFile;
    use IllError::*;

    const TAB: char = ' ';
    const NEWLINE: char = '\n';
    const REGISTER_DEF: char = '+';
    const DEF_END: char = ';';


    // instructions
    const INST_DEF: char = '$';

    const INST_PARAM_BEGIN: char = '(';
    const INST_PARAM_END: char = ')';

    const INST_CODES_BEGIN: char = '{';
    const INST_CODES_END: char = '}';

    // comments
    const COMMENT_SINGLE_LINE: char = '>';

    #[derive(Default, Debug, Clone)]
    pub struct Register {
        pub identifier: String,
        pub value: usize,
        pub is_variable: bool,
    }

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
                file: self.file.try_clone().expect("Failed to copy file..."),
            }
        }
    }

    #[derive(Default, Debug, Clone, Copy)]
    pub struct ReadHead {
        column: i32,
        line: i32,
    }

    #[derive(Debug)]
    pub enum IllError {
        RegisterRedefinition(ReadHead, String, Option<String>),
        NoRegistersFound(EnhancedFile),
        UnexpectedCharacter(ReadHead, char, Option<String>),
        InstructionRedefinition(ReadHead, String),
        UnknownOpCode(ReadHead, String),
        InvalidOpCodeArguments(ReadHead, String),
        OpCodeArgumentMismatch(ReadHead, String, i32, i32),
        NoMainInstruction(),
        OpCodeInvalidArgument(ReadHead, ExpressionType, String),
        // wanted, got
        OpCodeInvalidContainerReference(ReadHead, ExpressionType, String, String),
        UnescapedStringLiteralIsContainer(ReadHead, String),
        NonExistentRegister(ReadHead, String),
        NonExistentInstruction(ReadHead, String),
        ImmutableRegister(ReadHead, String),

    }

    impl Error for IllError {
        fn description(&self) -> &str {
            match *self {
                RegisterRedefinition(_, _, _) => "A Register redefinition was attempted.",
                NoRegistersFound(_) => "No Register definitions found.",
                UnexpectedCharacter(_, _, _) => "Unexpected character found.",
                InstructionRedefinition(_, _) => "A instruction redefinition was attempted.",
                UnknownOpCode(_, _) => "An unknown OpCode was used.",
                InvalidOpCodeArguments(_, _) => "An invalid instruction for an OpCode was found.",
                OpCodeArgumentMismatch(_, _, _, _) => "OpCode has too few or many arguments.",
                NoMainInstruction() => "No Main Instruction was found.",
                OpCodeInvalidArgument(_, _, _) => "Argument mismatch in OpCode.",
                OpCodeInvalidContainerReference(_, _, _, _) => "Container mismatch in OpCode.",
                UnescapedStringLiteralIsContainer(_, _) => "Expected String literal is also a container.",
                NonExistentRegister(_, _) => "Register does not exist.",
                NonExistentInstruction(_, _) => "Instruction does not exist.",
                ImmutableRegister(_, _) => "Register cannot be mutated.",
            }
        }
    }

    impl IllError {
        pub fn name(&self) -> String {
            String::from(match *self {
                RegisterRedefinition(_, _, _) => "Register Redefinition",
                NoRegistersFound(_) => "No Register Found",
                UnexpectedCharacter(_, _, _) => "Unexpected Character",
                InstructionRedefinition(_, _) => "Instruction Redefinition",
                UnknownOpCode(_, _) => "Unknown OpCode",
                InvalidOpCodeArguments(_, _) => "Invalid OpCode Instruction",
                OpCodeArgumentMismatch(_, _, _, _) => "OpCode Argument Length Mismatch",
                NoMainInstruction() => "No Main Instruction",
                OpCodeInvalidArgument(_, _, _) => "Argument Mismatch",
                OpCodeInvalidContainerReference(_, _, _, _) => "Container Mismatch",
                UnescapedStringLiteralIsContainer(_, _) => "Unescaped String Literal Misinterpreted",
                NonExistentRegister(_, _) => "Non-Existent Register",
                NonExistentInstruction(_, _) => "Non-Existent Instruction",
                ImmutableRegister(_, _) => "The Register is immutable."
            })
        }
    }

    impl Display for IllError {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            fn fmt_rh(rh: &ReadHead) -> String {
                format!("[{}:{}]", rh.line, rh.column)
            }
            fn rr_ext(n: &Option<String>) -> String {
                if n.is_some() { format!("(Shadowed by a {}.)", n.as_ref().unwrap()) } else { "".to_string() }
            }
            match *self {
                RegisterRedefinition(ref rh, ref name, ref e_type) => write!(
                    f,
                    "Err@{} => The Register named \"{}\" already exists! {}",
                    fmt_rh(rh),
                    name,
                    rr_ext(e_type)
                ),
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
                        "Err@{} => Found unexpected character {:?}{}",
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
                        "Err@{} => \"{}\"; invalid amount of arguments. Expected {}, but received {}.",
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
                OpCodeInvalidContainerReference(ref rh, ref e_type, ref got, ref msg) => {
                    write!(f, "Err@{} => Expected a {}, but got \"{}\" instead: {}.", fmt_rh(rh), e_type.name(), got, msg)
                }
                UnescapedStringLiteralIsContainer(ref rh, ref got) => write!(f, "Err@{} => Found an unescaped String literal that is also a container (register / variable). Try using \"{}\".", fmt_rh(rh), got),
                NonExistentRegister(ref rh, ref name) => write!(f, "Err@{} => The container {} does not exist globally nor locally.", fmt_rh(rh), name),
                NonExistentInstruction(ref rh, ref name) => write!(f, "Err@{} => The instruction {} does not exist.", fmt_rh(rh), name),
                ImmutableRegister(ref rh, ref name) => write!(f, "Err@{} => The register modified here {:?} is immutable.", fmt_rh(rh), name)
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

    #[derive(Default, Debug, Clone)]
    pub struct Instruction {
        pub name: String,
        codes: Vec<OpCode>,
        pub scope: Vec<Register>,
        arguments: Vec<String>,
        is_main: bool,
    }

    impl Instruction {
        fn new_default() -> Instruction {
            let mut scope: Vec<Register> = Vec::new();
            scope.push(Register {
                identifier: "res".to_string(),
                value: 0,
                is_variable: true,
            });
            Instruction { scope, ..Instruction::default() }
        }
        fn new(name: String, codes: Vec<OpCode>, mut scope: Vec<Register>, arguments: Vec<String>, is_main: bool) -> Instruction {
            scope.push(Register {
                identifier: "res".to_string(),
                value: 0,
                is_variable: true,
            });
            Instruction { name, codes, scope, arguments, is_main }
        }

        fn find_scoped_register(&self, name: String) -> Option<&Register> {
            self.scope.iter().find(|&x| x.identifier == name)
        }
        fn does_scoped_register_exist(&self, name: String) -> bool {
            self.find_scoped_register(name).is_some()
        }


        pub fn c_execute(&mut self, debug: bool, registers: &mut Vec<Register>, o_insts: Vec<Instruction>, c_scope: &mut Vec<Register>) -> Result<usize, IllError> {
            for opcode in &self.codes {
                let res = opcode.execute(debug, registers, o_insts.clone(), c_scope);
                if res.is_err() {
                    return Err(res.err().unwrap());
                }
            }
            let res_var = c_scope.iter().find(|x| x.identifier.to_lowercase() == String::from("res")).unwrap();
            Ok(res_var.value)
        }

        pub fn execute(&mut self, debug: bool, registers: &mut Vec<Register>, o_insts: Vec<Instruction>) -> Result<(), IllError> {
            for opcode in &self.codes {
                let res = opcode.execute(debug, registers, o_insts.clone(), &mut self.scope);

                if res.is_err() {
                    return res;
                }
            }
            Ok(())
        }
    }

    #[derive(Default)]
    pub struct Interpreter {
        pub debug: bool,
        files: Vec<EnhancedFile>,
        preamble: Vec<EnhancedFile>,
        opcodes: Vec<OpCode>,
        // valid opcodes
        pub registers: Vec<Register>,
        pub instructions: Vec<Instruction>,
    }

    #[derive(Default)]
    struct InstSwitchBox {
        is_reading_definition: bool,
        is_reading_arguments: bool,
        is_reading_codes: bool,
    }

    fn dump_all_until_any(head: &mut ReadHead, it: &mut Peekable<Chars>, ch: Vec<char>) {
        traverse_read(head, read_all_until_any(it, ch));
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

    // because fuck iterators
    // im sorry i love u iterator <3
    fn read_all_until_any(it: &mut Peekable<Chars>, ch: Vec<char>) -> (i32, i32, String) {
        let z = it.take_while(|c| ch.contains(c)).collect::<String>();
        let nl = newlines(&z);
        (
            nl,
            z.len() as i32 - nl,
            z,
        )
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

        pub fn new(debug: bool, sources: Vec<NamedFile>, preamble: Vec<NamedFile>, opcodes: Vec<OpCode>) -> Interpreter {
            if debug {
                println!("Making Interpreter with opcodes {:?}", opcodes);
            }
            Interpreter {
                opcodes,
                debug,
                preamble: preamble
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
                            println!("[:] content = `{:?}`", content);
                        }
                        EnhancedFile {
                            filename: nf.name.clone(),
                            file: clone,
                            content,
                        }
                    })
                    .collect(),
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
                            println!("[:] content = `{:?}`", content);
                        }
                        EnhancedFile {
                            filename: nf.name.clone(),
                            file: clone,
                            content,
                        }
                    })
                    .collect(),
                ..Default::default()
            }
        }

        fn find_register(&self, name: String) -> Option<&Register> {
            self.registers.iter().find(|x: &&Register| x.identifier == name)
        }

        fn does_register_exist(&self, name: String) -> bool {
            self.find_register(name).is_some()
        }

        fn find_instruction(&self, name: String) -> Option<&Instruction> {
            self.instructions.iter().find(
                |x: &&Instruction| x.name == name,
            )
        }

        fn does_instruction_exist(&self, name: String) -> bool {
            self.find_instruction(name).is_some()
        }

        fn parse_code(&self, rh: ReadHead, inst: &Instruction, insts: &Vec<Instruction>, code: String) -> Result<OpCode, IllError> {
            fn sanitize(str: String) -> String {
                str.replace("\"", "")
            }

            let mut pat = Pcre::compile(r#"('.*?'|".*?"|\S+)"#).unwrap();
            let data = pat.matches(&*code).map(|m| m.group(0)).collect::<Vec<_>>();
            let code_name = data[0].to_string();
            let nls = newlines(&code) as usize;
            let error_rh = rh.new_by(-(nls as i32), ((-rh.column) + code.len() as i32));
            if !self.does_opcode_exist(code_name.clone()) {
                return Err(IllError::UnknownOpCode(
                    error_rh,
                    sanitize(data[0].to_string()),
                ));
            }
            let opcode = self.find_opcode(code_name.clone()).unwrap().clone();
            if (data.len() - 1) != opcode.arguments.len() {
                return Err(IllError::OpCodeArgumentMismatch(
                    error_rh,
                    sanitize(data[0].to_string()),
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

            fn is_container(instruc: &Instruction, int: &Interpreter, ctx: String) -> bool {
                int.does_register_exist(ctx.clone()) || instruc.does_scoped_register_exist(ctx)
            }

            fn strip_quotes(str: String) -> String {
                str.replace("\"", "")
            }

            let mut exp_args = opcode.arguments.clone();
            let mut act_args: Vec<ExpressionType> = Vec::new();
            for i in 0..exp_args.len() {
                let expected = exp_args[i].clone().into();
                let ref argument = data[i + 1].to_string();
                if self.debug {
                    println!("arg = {}, expected = {:?}", argument, expected);
                }

                match expected {
                    ExpressionType::IntegerLiteral(_) => {
                        if inst.does_scoped_register_exist(argument.clone()) {
                            let reg = inst.find_scoped_register(argument.clone());
                            act_args.push(ExpressionType::IntegerLiteral(reg.unwrap().value as usize));
                        } else if self.does_register_exist(argument.clone()) {
                            let reg = self.find_register(argument.clone());
                            act_args.push(ExpressionType::IntegerLiteral(reg.unwrap().value as usize));
                        } /*else if !is_arg_literal(argument.clone()) {
                                return Err(OpCodeInvalidArgument(
                                    error_rh,
                                    r_literal(0),
                                    argument.clone()
                                ));
                        } */ else {
                                // See if I can do this by Reference? (looks to be too complicated, I don't want to have to use refcells)
                                act_args.push(ExpressionType::IntegerLiteral(argument.parse::<usize>().unwrap()));
                        }
                    }

                    ExpressionType::StringLiteral(_) => {
                        if is_container(inst, self, argument.clone()) {
                            return Err(UnescapedStringLiteralIsContainer(
                                error_rh,
                                argument.clone()
                            ));
                        } else if !is_arg_string(argument.clone()) {
                            return Err(OpCodeInvalidArgument(
                                error_rh,
                                s_literal(),
                                argument.clone()
                            ));
                        } else {
                            act_args.push(ExpressionType::StringLiteral(strip_quotes(argument.clone())));
                        }
                    }

                    ExpressionType::ContainerReference(_) => {
                        act_args.push(ExpressionType::ContainerReference(argument.clone()));
                    }
                    ExpressionType::RegisterReference(_) => {
                        act_args.push(ExpressionType::RegisterReference(argument.clone()));
                    }

                    ExpressionType::VariableReference(_) => {
                        act_args.push(ExpressionType::VariableReference(argument.clone()));
                    }
                    ExpressionType::InstructionReference(_, _) => {
                        let z = insts.iter().find(|x| x.name == argument.clone());
                        if z.is_some() {
                            act_args.push(ExpressionType::InstructionReference(argument.clone(), z.unwrap().arguments.clone()));
                        } else {
                            return Err(IllError::NonExistentInstruction(error_rh, argument.clone()));
                        }
                    }
                }
            }
            let has_result = act_args.iter().find(|x| x.name() == "res".to_string()).is_some();
            let result = if has_result {
                Some(0 as usize)
            } else {
                None
            };

            Ok(OpCode {
                name: code_name,
                arguments: act_args,
                location: Some(error_rh),
            })
        }

        fn scan_instructions(&mut self, preamble: bool) -> Result<(), IllError> {
            fn read_inst_def(it: &mut Peekable<Chars>) -> (i32, i32, String) {
                read_until(it, vec![INST_PARAM_BEGIN])
            }

            for e_file in if preamble { &self.preamble } else { &self.files } {
                let mut it = e_file.content.chars().peekable();
                let mut head: ReadHead = ReadHead::new();
                let mut cur_inst: Instruction = Instruction::new_default();
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
                            let register_name = traverse_read(&mut head, read_inst_def(it.by_ref()));
                            cur_inst.name = register_name;
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
                                        *it.peek().unwrap(), // TODO: Change back into x if we need to.
                                        Some(format!(
                                            " expecting instruction code beginning \"{}\".",
                                            INST_CODES_BEGIN
                                        )),
                                    ));
                                }
                            dump_until(&mut head, it.by_ref(), vec![INST_CODES_BEGIN]);
                            while it.peek().is_some() && *it.peek().unwrap() != INST_CODES_END {
                                if it.clone().collect::<String>().contains(COMMENT_SINGLE_LINE) {
                                    let x = traverse_read(&mut head, read_all_until_any(it.by_ref(), vec![NEWLINE, TAB, COMMENT_SINGLE_LINE]));
                                    if x.contains(COMMENT_SINGLE_LINE) {
                                        dump_until(&mut head, it.by_ref(), vec![NEWLINE]);
                                    }
                                }

                                let chars = it.clone().collect::<String>();
                                if *it.peek().unwrap() == COMMENT_SINGLE_LINE {
                                    dump_until(&mut head, it.by_ref(), vec![NEWLINE]);
                                }
                                if !any_exists_until(
                                    &mut it.clone(),
                                    vec![DEF_END],
                                    vec![INST_CODES_END],
                                )
                                    {
                                        // break because no codes
                                        break;
                                    }

                                let raw_code = traverse_read(
                                    &mut head,
                                    read_until_spare_ws(it.by_ref(), vec![DEF_END]),
                                );
                                let code = String::from(raw_code.trim());

                                let res = self.parse_code(head.clone(), &cur_inst, &self.instructions, code.clone());
                                if res.is_err() {
                                    return Err(res.err().unwrap());
                                }
                                cur_inst.codes.push(res.ok().unwrap());
                                if self.debug {
                                    println!("found code {:?}", code);
                                }
                            }
                            cur_inst_sb.is_reading_codes = false;
                            if self.does_instruction_exist(cur_inst.name.clone()) {
                                return Err(IllError::InstructionRedefinition(
                                    head.new_by(0, -(cur_inst.name.len() as i32)),
                                    cur_inst.name,
                                ));
                            }
                            self.instructions.push(cur_inst);
                            cur_inst = Instruction::new_default();
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
            if self.debug {
                println!("insts = {:?}", self.instructions);
            }
            if !preamble {
                // inst.execute(debug, &self.registers, &self.instructions);
                let inst_clone = self.instructions.clone();
                let mut_inst: &mut Vec<Instruction> = self.instructions.as_mut();
                let inst = mut_inst.iter_mut().find(|x| x.is_main).unwrap();
                inst.execute(self.debug, &mut self.registers, inst_clone)
            } else {
                Ok(())
            }
        }

        fn create_registers(&mut self) -> Result<(), IllError> {
            for e_file in &self.files {
                let mut iter = e_file.content.chars().peekable();
                let mut head: ReadHead = ReadHead::new();
                let mut has_found_registers: bool = false;
                while let Some(x) = iter.next() {
                    head.advance(x);
                    if !x.is_whitespace() {
                        if x == REGISTER_DEF {
                            has_found_registers = true;
                            while iter.peek().is_some() && *iter.peek().unwrap() != NEWLINE {
                                let register_name = traverse_read(
                                    &mut head,
                                    read_until(iter.by_ref(), vec![DEF_END]),
                                );
                                if self.does_register_exist(register_name.clone()) {
                                    let err_str = register_name.clone();
                                    return Err(RegisterRedefinition(head, err_str, None));
                                }
                                self.registers.push(Register {
                                    identifier: register_name,
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

        pub fn begin_parsing(&mut self) -> Option<IllError> {
            self.scan_instructions(true);

            let res = self.create_registers();
            if res.is_err() {
                return res.err();
            }

            let debug = self.debug;

            let res = self.scan_instructions(false);
            if res.is_err() {
                return res.err();
            }

            if self.debug {
                println!("end_registers = {:?}", self.registers);
                for inst in &self.instructions {
                    println!("{}'s registers: {:?}", inst.name, inst.scope);
                }
            }

            None
        }
    }
}