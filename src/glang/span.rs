use super::scope::{Scope, ScopeValue};

#[derive(Debug, Clone, Default)]
pub struct MultiVariableOptions {
    // The delimeter between items emitted from this (multi) variable
    delimeter: char,
    // Should the delimeter be emitted after the last item ?
    trailing: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum Instruction<'t> {
    Newline,

    Indent(u16),

    Literal(&'t str),

    Evaluate(&'t str),
}

#[derive(Clone, Debug)]
pub struct TemplateSpan<'t> {
    instructions: Vec<Instruction<'t>>,
}

impl<'t> TemplateSpan<'t> {
    pub fn empty() -> Self {
        Self {
            instructions: vec![],
        }
    }

    pub fn compile(content: &'t str) -> TemplateSpan {
        let mut span = TemplateSpan {
            instructions: vec![],
        };

        #[derive(Clone, Copy, Debug)]
        enum State {
            Indenting,
            Literal,
            Variable,
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
            let num_chars = chars.len();

            let mut i = 0;

            while i < num_chars {
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
                            State::Variable
                        } else {
                            State::Literal
                        };
                        // NOTE: % is included in start in case of State::Variable
                        start = index;
                    }

                    (State::Literal, c) => {
                        if c == '%' {
                            if (index > start) {
                                span.instructions
                                    .push(Instruction::Literal(&line[start..index]));
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

                            span.instructions
                                .push(Instruction::Evaluate(&line[start..index]));

                            state = State::Literal;
                            start = index + percentage_size;
                        } else if c == '/' {
                            // options
                            let (_, delim) = chars[i + 1];
                            let trailing = chars[i + 2].1 == '+';

                            // ignore the starting %
                            let var_start = start + percentage_size;

                            if trailing {
                                assert_eq!(chars[i + 3].1, '%');
                                // TODO: Very unreadable
                                start = chars[(i + 4).min(num_chars - 1)].0;
                                i = i + 4;
                            } else {
                                assert_eq!(chars[i + 2].1, '%');
                                start = chars[(i + 3).min(num_chars - 1)].0;
                                i = i + 3;
                            }

                            span.instructions
                                .push(Instruction::Evaluate(&line[var_start..index]));

                            state = State::Literal;
                        };
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
            } else if let State::Indenting = state {
                if indent > 0 {
                    span.instructions.push(Instruction::Indent(indent));
                }
            } else {
                // syntax error
                panic!("Syntax error in TemplateSpan {:?}", state);
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
                &Instruction::Evaluate(variable) => {
                    let scope_val = scope.map.get(variable).unwrap_or_else(|| {
                        panic!("Unknown variable %{}%", variable);
                    });
                    match scope_val {
                        ScopeValue::Text(text) => print!("{}", text),
                        ScopeValue::Expand(expander) => expander.expand(base_indent + line_indent),
                    }
                }
            }
        }
    }
}

pub fn do_indent(size: u16) {
    // TODO: optimize
    for _ in 0..size {
        print!(" ");
    }
}
