use super::scope::{Scope, ScopeValue};

#[derive(Clone, Copy, Debug)]
pub enum Instruction<'t> {
    Newline,

    Indent(u16),

    Literal(&'t str),

    EvaluateSingle(&'t str),

    EvaluateMultiple(&'t str),
}

#[derive(Clone, Debug)]
pub struct TemplateSpan<'t> {
    instructions: Vec<Instruction<'t>>,
}

impl<'t> TemplateSpan<'t> {
    pub fn compile(content: &'t str) -> TemplateSpan {
        let mut span = TemplateSpan {
            instructions: vec![],
        };

        #[derive(Clone, Copy)]
        enum State {
            Indenting,
            Literal,
            VariableDetected,
            VariableSingle,
            VariableMutiple,
        }

        let mut is_tail = false;

        for line in content.lines() {
            if is_tail {
                span.instructions.push(Instruction::Newline);
            } else {
                is_tail = true;
            }

            let mut indent = 0;
            let mut start = 0;
            let mut state = State::Indenting;

            let mut chars = line.char_indices().collect::<Vec<_>>();

            let mut i = 0;
            while i < chars.len() {
                let (index, c) = chars[i];

                match (state, c) {
                    (State::Indenting, c) if c.is_whitespace() => {
                        // current +=
                        indent += 1;
                    }
                    (State::Indenting, c) => {
                        if indent > 0 {
                            span.instructions.push(Instruction::Indent(indent));
                        }
                        state = if c == '%' {
                            State::VariableDetected
                        } else {
                            State::Literal
                        };
                        start = index;
                    }

                    (State::Literal, c) => {
                        if c == '%' {
                            if (index > start) {
                                span.instructions
                                    .push(Instruction::Literal(&line[start..index]));
                            }
                            state = State::VariableDetected;
                            start = index;
                        }
                    }

                    (State::VariableDetected, c) => {
                        if c == '%' {
                            state = State::VariableMutiple;
                            // Index of next char
                            start = index + '%'.len_utf16();
                        } else {
                            state = State::VariableSingle;
                            start = index;
                        }
                    }

                    (State::VariableSingle, c) => {
                        if c == '%' {
                            span.instructions
                                .push(Instruction::EvaluateSingle(&line[start..index]));

                            state = State::Literal;
                            start = index + c.len_utf16();
                        }
                    }

                    (State::VariableMutiple, c) => {
                        let next = chars.get(i + 1);
                        if let Some(&(next_index, p)) = next {
                            if c == '%' && p == '%' {
                                span.instructions
                                    .push(Instruction::EvaluateMultiple(&line[start..index]));
                                state = State::Literal;
                                start = next_index + '%'.len_utf16();
                                i += 1;
                            }
                        }
                    }

                    _ => (),
                }

                i += 1;
            }

            if let State::Literal = state {
                let lit = &line[start..];
                if !lit.is_empty() {
                    span.instructions.push(Instruction::Literal(lit));
                }
            } else {
                // syntax error
                panic!("Syntax error in TemplateSpan");
            }
        }

        span
    }

    pub fn print(&self, base_indent: u16, scope: Scope) {
        let mut line_indent = 0;
        for inst in self.instructions.iter() {
            match inst {
                Instruction::Newline => {
                    print!("\n");
                    line_indent = 0;
                    do_indent(base_indent);
                }
                &Instruction::Indent(size) => {
                    line_indent = size;
                    do_indent(size);
                }
                Instruction::Literal(val) => {
                    print!("{}", val);
                }
                &Instruction::EvaluateSingle(variable) => {
                    let scope_val = scope.map.get(variable).unwrap();
                    match scope_val {
                        ScopeValue::Text(text) => print!("{}", text),
                        ScopeValue::Expand(expander) => expander.expand(base_indent + line_indent),
                    }
                }
                &Instruction::EvaluateMultiple(variable) => {
                    let scope_val = scope.map.get(variable).unwrap();
                    match scope_val {
                        ScopeValue::Expand(expander) => expander.expand(base_indent + line_indent),
                        ScopeValue::Text(..) => {
                            panic!("a %%multiple%% variable cannot have a single text value")
                        }
                    }
                }
            }
        }
    }
}

fn do_indent(size: u16) {
    // TODO: optimize
    for _ in 0..size {
        print!(" ");
    }
}

