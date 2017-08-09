

pub mod ill {
    use std::fs::File;
    use std::io::Read;
    use std::iter::Peekable;
    use std::str::Chars;
    use std::error::Error;
    use std::fmt;
    use std::fmt::{Display, Formatter};

    use opcodes::ill::OpCode;

    use NamedFile;
    use IllError::*;

    const NEWLINE: char = '\n';
    const STACK_DEF: char = '+';
    const DEF_END: char = ';';


    // instructions
    const INST_DEF: char = '$';

    const INST_PARAM_BEGIN: char = '(';
    const INST_PARAM_END: char = ')';

    const INST_CODES_BEGIN: char = '{';
    const INST_CODES_END: char = '}';



    #[derive(Default, Debug)]
    pub struct Stack {
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
                file: self.file.try_clone().expect("Faield to copy file..."),
            }
        }
    }

    #[derive(Default, Debug, Clone)]
    pub struct ReadHead {
        column: u32,
        line: u32,
    }

    #[derive(Debug)]
    pub enum IllError {
        StackRefinition(ReadHead, String),
        NoStacksFound(EnhancedFile),
        UnexpectedCharacter(ReadHead, char),
        InstructionRedefinition(ReadHead, String),
        UnknownOpCode(ReadHead, String),
    }

    impl Error for IllError {
        fn description(&self) -> &str {
            match *self {
                StackRefinition(_, _) => "A stack redefinition was attempted.",
                NoStacksFound(_) => "No stack definitions found.",
                UnexpectedCharacter(_, _) => "Unexpected character found.",
                InstructionRedefinition(_, _) => "A instruction redefinition was attempted.",
                UnknownOpCode(_, _) => "An unknown OpCode was used!",
            }
        }
    }

    impl IllError {
        pub fn name(&self) -> String {
            String::from(match *self {
                StackRefinition(_, _) => "Stack Redefinition",
                NoStacksFound(_) => "No Stack Found",
                UnexpectedCharacter(_, _) => "Unexpected Character",
                InstructionRedefinition(_, _) => "Instruction Redefinition",
                UnknownOpCode(_, _) => "Unknown OpCode",
            })
        }
    }

    impl Display for IllError {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            fn fmt_rh(rh: &ReadHead) -> String {
                format!("[{}:{}]", rh.line, rh.column)
            }
            match *self {
                StackRefinition(ref rh, ref name) => {
                    write!(
                        f,
                        "Err@{} => The stack named \"{}\" already exists!",
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
                NoStacksFound(ref e_file) => {
                    write!(
                        f,
                        "Cannot find a stack definition for {:?}.",
                        e_file.filename
                    )
                }
                UnexpectedCharacter(ref rh, ref ch) => {
                    write!(
                        f,
                        "Err@{} => Found unexpected character {}.",
                        fmt_rh(rh),
                        ch
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
            }
        }
    }

    impl ReadHead {
        fn new() -> ReadHead {
            ReadHead { line: 1, column: 1 }
        }
        fn advance_by(&mut self, line: u32, col: u32) {
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
        arguments: Vec<String>,
        is_main: bool,
    }

    #[derive(Default)]
    pub struct Interpreter {
        debug: bool,
        files: Vec<EnhancedFile>,
        opcodes: Vec<OpCode>, // valid opcodes
        stacks: Vec<Stack>,
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

    fn read_until_spare_ws(it: &mut Peekable<Chars>, ch: Vec<char>) -> (u32, String) {
        let z = it.take_while(|c| !ch.contains(c)).collect::<String>();
        (
            ((z.len() + 1) - z.chars().filter(|x| *x == NEWLINE).count() as usize) /* Compensate for missing ';' */ as u32,
            z.chars().collect::<String>(),
        )
    }

    fn read_until(it: &mut Peekable<Chars>, ch: Vec<char>) -> (u32, String) {
        let z = it.take_while(|c| !ch.contains(c)).collect::<String>();
        (
            ((z.len() + 1) - z.chars().filter(|x| *x == NEWLINE).count() as usize) /* Compensate for missing ';' */ as u32,
            z.chars().filter(|c| !c.is_whitespace()).collect::<String>(),
        )
    }

    fn any_exists_until(it: &mut Peekable<Chars>, exists: Vec<char>, until: Vec<char>) -> bool {
        let (_, data) = read_until(it, until);
        data.chars().find(|x| exists.contains(x)).is_some()
    }

    fn traverse_read(head: &mut ReadHead, data: (u32, String)) -> String {
        let (trav, dat) = data;
        // println!("traversing {} for \"{}\" [{}]", trav, dat, dat.len());
        head.advance_by(dat.chars().filter(|x| *x == NEWLINE).count() as u32, trav);
        dat
    }


    impl Interpreter {
        fn does_opcode_exist(&self, name: &String) -> bool {
            self.opcodes
                .iter()
                .find(|x: &&OpCode| x.name == name.clone())
                .is_some()
        }

        pub fn new(debug: bool, sources: Vec<NamedFile>, opcodes: Vec<OpCode>) -> Interpreter {
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

        fn find_stack(&self, name: String) -> Option<&Stack> {
            self.stacks.iter().find(|x: &&Stack| x.identifier == name)
        }

        fn does_stack_exist(&self, name: String) -> bool {
            self.find_stack(name).is_some()
        }

        fn find_instruction(&self, name: String) -> Option<&Instruction> {
            self.instructions.iter().find(
                |x: &&Instruction| x.name == name,
            )
        }

        fn does_instruction_exist(&self, name: String) -> bool {
            self.find_instruction(name).is_some()
        }

        fn parse_code(&self, rh: ReadHead, code: String) -> Result<OpCode, IllError> {
            let data: Vec<String> = code.split(' ').map(String::from).collect::<Vec<String>>();
            let code_name = data[0].clone();
            if self.does_opcode_exist(&code_name) {
                return Err(IllError::UnknownOpCode(rh, data[0].clone()));
            }
            Ok(OpCode::new_str(code_name))
        }

        fn scan_instructions(&mut self) -> Result<(), IllError> {

            fn read_inst_def(it: &mut Peekable<Chars>) -> (u32, String) {
                let (trav, st) = read_until(it, vec![INST_PARAM_BEGIN]);
                (trav, st)
            }

            for e_file in &self.files {
                let mut it = e_file.content.chars().peekable();
                let mut head: ReadHead = ReadHead::new();
                let mut cur_inst: Instruction = Default::default();
                let mut cur_inst_sb: InstSwitchBox = Default::default();
                while let Some(x) = it.next() {
                    head.advance(x);
                    if x == INST_DEF {
                        if cur_inst_sb.is_reading_definition {
                            return Err(UnexpectedCharacter(head, x));
                        } else {
                            cur_inst_sb.is_reading_definition = true;
                        }
                        if cur_inst_sb.is_reading_definition {
                            cur_inst.is_main = *it.peek().unwrap() == INST_DEF;
                            let stack_name = traverse_read(&mut head, read_inst_def(it.by_ref()));
                            cur_inst.name = stack_name;
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
                            dump_until(&mut head, it.by_ref(), vec![INST_CODES_BEGIN]);
                            while *it.peek().unwrap() != INST_CODES_END {
                                if !any_exists_until(
                                    &mut it.clone(),
                                    vec![DEF_END],
                                    vec![INST_CODES_END],
                                )
                                {
                                    break;
                                }
                                let code = traverse_read(
                                    &mut head,
                                    read_until_spare_ws(it.by_ref(), vec![DEF_END]),
                                ).chars()
                                    .filter(|x| *x != NEWLINE)
                                    .collect::<String>();
                                let t_code = code.trim();
                                let res = self.parse_code(head.clone(), String::from(t_code));
                                if res.is_err() {
                                    return Err(res.err().unwrap());
                                }
                                cur_inst.codes.push(res.ok().unwrap());
                                println!("found code \"{}\"", t_code);
                            }
                            cur_inst_sb.is_reading_codes = false;
                            if self.does_instruction_exist(cur_inst.name.clone()) {
                                return Err(IllError::InstructionRedefinition(head, cur_inst.name));
                            }
                            self.instructions.push(cur_inst);
                            cur_inst = Default::default();
                            cur_inst_sb = Default::default();
                        }
                    }
                }
            }
            if self.instructions.len() == 1 {
                self.instructions[0].is_main = true;
            }
            println!("insts = {:?}", self.instructions);
            Ok(())
        }

        fn create_stacks(&mut self) -> Result<(), IllError> {

            for e_file in &self.files {
                let mut iter = e_file.content.chars().peekable();
                let mut head: ReadHead = ReadHead::new();
                let mut has_found_stacks: bool = false;
                while let Some(x) = iter.next() {
                    head.advance(x);
                    if !x.is_whitespace() {
                        if x == STACK_DEF {
                            has_found_stacks = true;
                            while iter.peek().is_some() && *iter.peek().unwrap() != NEWLINE {
                                let (trav, stack_name) =
                                    read_until(iter.by_ref(), vec![DEF_END, NEWLINE]);
                                if self.does_stack_exist(stack_name.clone()) {
                                    let err_str = stack_name.clone();
                                    return Err(StackRefinition(head, err_str));
                                }
                                head.advance_by(0, trav);
                                self.stacks.push(Stack {
                                    identifier: stack_name,
                                    is_variable: false,
                                    ..Default::default()
                                });
                                continue;
                            }
                        }
                    }
                }
                if !has_found_stacks {
                    return Err(NoStacksFound(e_file.clone()));
                } else if self.debug {
                    println!("Found stacks: {:?}", self.stacks);
                }
            }
            Ok(())
        }

        pub fn begin_parsing(&mut self) -> Result<(), IllError> {
            let res = self.create_stacks();
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