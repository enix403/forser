use super::scope::{AssemblyContext, Scope, ScopeValue};
use std::io::{self, Write};

#[derive(Debug, Clone, Default)]
pub struct ExpandOptions {
    // The delimeter between items emitted from this (multi) variable
    delimeter: Option<char>,
    // Should the delimeter be emitted after the last item ?
    trailing: bool,
}

#[derive(Clone, Debug)]
pub enum Instruction<'t> {
    Newline,

    Indent(u16),

    Literal(&'t str),

    Evaluate { var: &'t str, opts: ExpandOptions },
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
            Closed,
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

            let mut i: usize = 0;

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
                                // .push(Instruction::Evaluate(&line[start..index]));
                                .push(Instruction::Evaluate {
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

                            span.instructions.push(Instruction::Evaluate {
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

    pub fn print<C: AssemblyContext>(
        &self,
        base_indent: u16,
        mut scope: Scope<C>,
    ) -> io::Result<()> {
        print_span_impl(self, base_indent, scope)
    }
}

fn print_span_impl<C: AssemblyContext>(
    span: &TemplateSpan,
    base_indent: u16,
    mut scope: Scope<C>,
) -> io::Result<()> {
    let mut line_indent = 0;
    for inst in span.instructions.iter() {
        match inst {
            Instruction::Newline => {
                scope.write("\n")?;
                line_indent = 0;
                scope.do_indent(base_indent)?;
            }
            Instruction::Indent(size) => {
                let size = *size;
                line_indent = size;
                scope.do_indent(size)?;
            }
            Instruction::Literal(val) => {
                scope.write(val)?;
            }
            Instruction::Evaluate { var, opts } => {
                // let scope_val = scope.map.get_mut(var).unwrap_or_else(|| {
                //     panic!("Unknown variable %{}%", var);
                // });

                let scope_val = scope.get_value(var);

                match scope_val {
                    // ScopeValue::Text(text) => write!(dest, "{}", text)?,
                    ScopeValue::Text(text) => scope.write(text)?,
                    ScopeValue::Expand(expander) => {
                        let total = expander.count();
                        let next_indent = line_indent + base_indent;

                        for index in 0..total {
                            if index != 0 {
                                if let Some(delim) = opts.delimeter {
                                    // TDOD: to string
                                    scope.write(delim.to_string().as_str())?;
                                }
                                scope.write("\n")?;
                                scope.do_indent(next_indent)?;
                            }

                            scope.with_context(|context| {
                                expander.expand_next(context, next_indent);
                            })
                        }

                        if opts.trailing && opts.delimeter.is_some() {
                            scope.write(opts.delimeter.unwrap().to_string().as_str())?;
                        }
                    }
                } // End match ScopeValue
            } /* End Match Instruction */
        }
    }

    Ok(())
}
