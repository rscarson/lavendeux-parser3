use crate::{
    error::Error,
    tokenizer::{Category, Rule, Token},
    IntoOwned,
};

/// Token queue with rewind
#[derive(Clone, Debug)]
pub struct Stack<'source> {
    // All tokens in the stack
    tokens: Vec<Token<'source>>,

    // Transactions, and the number of tokens in the stack at the time of each
    cur_len: Vec<usize>,

    // The furthest position we tried to match against
    error_pos: usize,

    // The set of rules we tried to match at error_pos
    could_expect: Vec<Rule>,
}
impl<'source> Stack<'source> {
    /// Creates a new stack
    pub fn new(mut tokens: Vec<Token<'source>>) -> Self {
        let len = tokens.len();
        tokens.reverse();
        Self {
            tokens,
            cur_len: vec![len],
            error_pos: len - 1,
            could_expect: vec![],
        }
    }
    pub fn len(&self) -> usize {
        *self.cur_len.last().unwrap()
    }

    pub fn depth(&self) -> usize {
        self.cur_len.len() - 1
    }

    pub fn expecting(&self) -> Vec<Category> {
        Category::from_ruleset(&self.could_expect)
    }

    pub fn error_token(&self) -> &Token<'source> {
        self.tokens.get(self.error_pos).unwrap()
    }

    /// Emit an error with the current state
    pub fn emit_err(&self) -> Error {
        Error::Syntax {
            expected: self.expecting(),
            found: self.error_token().clone().into_owned(),
        }
    }

    fn try_update_error_pos(&mut self, rules: &[Rule]) {
        let len = self.len();
        if self.error_pos > len && len > 0 {
            self.error_pos = len - 1;
            self.could_expect.clear();
        }

        if self.error_pos + 1 == len {
            self.could_expect.extend(rules);
        }
    }

    fn take_one(&mut self) {
        match self.len() {
            0 => {}
            _ => {
                *self.cur_len.last_mut().unwrap() -= 1;
            }
        }
    }

    fn unborrow_err(&mut self) -> usize {
        match self.cur_len.len() {
            1 => 1,
            _ => self.cur_len.pop().unwrap(),
        }
    }

    fn unborrow_ok(&mut self) -> usize {
        let l = self.unborrow_err();
        *self.cur_len.last_mut().unwrap() = l;
        l
    }

    fn borrow(&mut self) {
        self.cur_len.push(self.len());
    }

    /// Remove a token from the stack
    /// Changes do not persist until `apply` or `revert` are called
    pub fn pop(&mut self) -> Option<Token<'source>> {
        if let Some(t) = self.tokens.get(self.len() - 1).cloned() {
            self.take_one();
            Some(t)
        } else {
            None
        }
    }

    /// Peek at the next token on the stack
    pub fn peek(&self) -> Option<&Token<'source>> {
        let len = self.len();
        match len {
            0 => None,
            _ => self.tokens.get(len - 1),
        }
    }

    /// Return a token, or error
    pub fn try_pop_a(&mut self, rules: &[Rule]) -> Result<Token<'source>, Error> {
        self.try_update_error_pos(rules);

        if let Some(t) = self.peek() {
            if t.is_a(rules) {
                return Ok(self.pop().unwrap());
            } else if t.rule() == Rule::Error {
                return Err(Error::UnrecognizedToken(self.pop().unwrap().into_owned()));
            }
        }

        self.revert_transaction();
        Err(self.emit_err())
    }

    /// Return a token, or error
    pub fn try_peek_a(&mut self, rules: &[Rule]) -> Result<&Token<'source>, Error> {
        self.try_update_error_pos(rules);
        if let Some(t) = self.peek() {
            if t.is_a(rules) {
                return Ok(t);
            } else if t.rule() == Rule::Error {
                return Err(Error::UnrecognizedToken(t.clone().into_owned()));
            }
        }

        Err(self.emit_err())
    }

    /// Apply pending changes to the stack
    pub fn apply_transaction(&mut self) {
        self.unborrow_ok();
    }

    /// Revert the stack to previous form
    pub fn revert_transaction(&mut self) {
        self.unborrow_err();
    }

    /// Start a new reversible transaction on the stack
    pub fn start_transaction(&mut self) {
        self.borrow();
    }
}
