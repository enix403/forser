// pub use super::expander::{AssemblyContext, Expander};
// use std::cell::RefCell;
// use std::collections::HashMap;
// use std::io;

// pub enum ScopeValue<'v, 't, C> {
//     Text(&'v str),
//     Expand(Box<dyn Expander<C> + 't>),
// }

// pub struct Scope<'v, 't, C> {
//     // TODO: 'static ?
//     pub map: RefCell<HashMap<&'static str, ScopeValue<'v, 't, C>>>,
//     context: RefCell<C>,
// }

// impl<'v, 't, C: AssemblyContext> Scope<'v, 't, C> {
//     pub fn new(context: C) -> Self {
//         Self {
//             map: RefCell::new(HashMap::new()),
//             context: RefCell::new(context),
//         }
//     }

//     pub fn add_text(mut self, name: &'static str, text: &'v str) -> Self {
//         self.map.borrow_mut().insert(name, ScopeValue::Text(text));
//         self
//     }

//     pub fn add_expander<E: Expander<C> + 't>(mut self, name: &'static str, expander: E) -> Self {
//         self.map
//             .borrow_mut()
//             .insert(name, ScopeValue::Expand(Box::new(expander)));
//         self
//     }

//     pub fn get_value(&self, name: &'static str) -> &mut ScopeValue<'v, 't, C> {
//         self.map.borrow_mut().get_mut(name).unwrap_or_else(|| {
//             panic!("Unknown variable %{}%", name);
//         })
//     }

//     pub fn write(&mut self, s: &str) -> io::Result<()> {
//         self.context.borrow_mut().write(s)
//     }

//     pub fn do_indent(&mut self, size: u16) -> io::Result<()> {
//         // TODO: optimize
//         for _ in 0..size {
//             self.write(" ")?;
//         }

//         Ok(())
//     }

//     pub fn with_context<F>(&self, func: F)
//     where
//         F: FnOnce(&mut C),
//     {
//         let context_ref = &mut *self.context.borrow_mut();
//         func(context_ref);
//     }
// }
