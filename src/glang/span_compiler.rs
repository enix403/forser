#[derive(Clone, Debug)]
struct ExpandOptions {}

#[derive(Clone, Debug)]
pub enum Instruction<'t> {
    Newline,

    Indent(u16),

    Literal(&'t str),

    Expand { var: &'t str, opts: ExpandOptions },
}

#[derive(Clone, Debug, Default)]
pub struct TemplateSpan<'t> {
    pub instructions: Vec<Instruction<'t>>,
}


/*
fn compile_span<'t>(content: &'t str) -> TemplateSpan<'t> {
    let mut instructions = vec![];

    #[derive(Clone, Copy, Debug)]
    enum State {
        Indenting,
        Literal,
        Variable,
        Closed,
    }

    let mut is_tail = false;

    for line in content.lines() {
        if is_tail {
            instructions.push(Instruction::Newline);
        } else {
            is_tail = true;
        }

        let mut indent = 0;
        let mut start = 0;
        let mut state = State::Indenting;

        let chars = line.char_indices().collect::<Vec<_>>();
        let num_chars = chars.len();

        let mut i: usize = 0;

        while i < num_chars {
            let (index, c) = chars[i];

            match (state, c) {
                (State::Indenting, c) if c.is_whitespace() => {
                    indent += 1;
                }
                (State::Indenting, c) => {
                    if indent > 0 {
                        instructions.push(Instruction::Indent(indent));
                    }
                    state = if c == '%' {
                        State::Variable
                    } else {
                        State::Literal
                    };
                    // NOTE: % is included in start in case of State::Variable
                    start = index;
                }

                (State::Literal, c) => {
                    if c == '%' {
                        if index > start {
                            instructions.push(Instruction::Literal(&line[start..index]));
                        }
                        state = State::Variable;
                        // NOTE: % is included in start
                        start = index;
                    }
                }

                (State::Variable, c) => {
                    let percentage_size = '%'.len_utf16();
                    if c == '%' {
                        // ignore the starting %
                        start += percentage_size;

                        instructions.push(Instruction::Expand {
                            var: &line[start..index],
                            opts: ExpandOptions::default(),
                        });

                        state = State::Literal;
                        start = index + percentage_size;
                    } else if c == '/' {
                        // options
                        let (_, delimeter) = chars[i + 1];
                        let trailing = chars[i + 2].1 == '+';

                        // ignore the starting %
                        let var_start = start + percentage_size;

                        // Update the start before as it might later become State::Closed
                        state = State::Literal;

                        {
                            let new_index = if trailing { i + 4 } else { i + 3 };

                            assert_eq!(chars[new_index - 1].1, '%');
                            if new_index >= num_chars {
                                state = State::Closed;
                            } else {
                                start = chars[new_index].0;
                            }
                            i = new_index;
                        }

                        instructions.push(Instruction::Expand {
                            var: &line[var_start..index],
                            opts: ExpandOptions {
                                delimeter: Some(delimeter),
                                trailing,
                            },
                        });
                    };
                }

                _ => (),
            }

            i += 1;
        }

        if let State::Closed = state {
            // ... do nothing ...
        } else if let State::Literal = state {
            let lit = &line[start..];
            if !lit.is_empty() {
                instructions.push(Instruction::Literal(lit));
            }
        } else if let State::Indenting = state {
            if indent > 0 {
                instructions.push(Instruction::Indent(indent));
            }
        } else {
            // syntax error
            panic!("Syntax error in TemplateSpan {:?}", state);
        }
    }

    TemplateSpan { instructions }
}

*/
