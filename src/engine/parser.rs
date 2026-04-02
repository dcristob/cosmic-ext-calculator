use super::CalcError;

pub struct Parser {
    input: Vec<char>,
    pos: usize,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            input: Vec::new(),
            pos: 0,
        }
    }

    /// Parse and evaluate an expression (no functions).
    pub fn parse(&mut self, input: &str) -> Result<f64, CalcError> {
        self.parse_with_functions(input, |name, _args| {
            Err(CalcError::InvalidExpression(format!(
                "Unknown function: {name}"
            )))
        })
    }

    /// Parse and evaluate with a function resolver callback.
    pub fn parse_with_functions<F>(&mut self, input: &str, func_eval: F) -> Result<f64, CalcError>
    where
        F: Fn(&str, &[f64]) -> Result<f64, CalcError>,
    {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(CalcError::InvalidExpression("Empty expression".into()));
        }
        self.input = trimmed.chars().collect();
        self.pos = 0;

        let result = self.expression(&func_eval)?;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            return Err(CalcError::InvalidExpression(format!(
                "Unexpected character: '{}'",
                self.input[self.pos]
            )));
        }
        Ok(result)
    }

    // ── Helpers ──────────────────────────────────────────────────

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.input.get(self.pos).copied();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn expect(&mut self, ch: char) -> Result<(), CalcError> {
        self.skip_whitespace();
        match self.peek() {
            Some(c) if c == ch => {
                self.advance();
                Ok(())
            }
            Some(c) => Err(CalcError::InvalidExpression(format!(
                "Expected '{ch}', found '{c}'"
            ))),
            None => Err(CalcError::InvalidExpression(format!(
                "Expected '{ch}', found end of input"
            ))),
        }
    }

    // ── Grammar ──────────────────────────────────────────────────

    // expression = term (('+' | '-') term)*
    fn expression<F>(&mut self, func_eval: &F) -> Result<f64, CalcError>
    where
        F: Fn(&str, &[f64]) -> Result<f64, CalcError>,
    {
        let mut left = self.term(func_eval)?;
        loop {
            self.skip_whitespace();
            match self.peek() {
                Some('+') => {
                    self.advance();
                    left += self.term(func_eval)?;
                }
                Some('-') => {
                    self.advance();
                    left -= self.term(func_eval)?;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    // term = power (('*' | '/' | '%' | '×' | '÷') power)*
    // '%' is modulus here only when followed by something that looks like a value
    fn term<F>(&mut self, func_eval: &F) -> Result<f64, CalcError>
    where
        F: Fn(&str, &[f64]) -> Result<f64, CalcError>,
    {
        let mut left = self.power(func_eval)?;
        loop {
            self.skip_whitespace();
            match self.peek() {
                Some('*') | Some('×') => {
                    self.advance();
                    let right = self.power(func_eval)?;
                    left *= right;
                }
                Some('/') | Some('÷') => {
                    self.advance();
                    let right = self.power(func_eval)?;
                    if right == 0.0 {
                        return Err(CalcError::DivisionByZero);
                    }
                    left /= right;
                }
                Some('%') => {
                    // '%' followed by digit/paren/space → modulus; otherwise break
                    // (postfix percentage is handled in the postfix() layer)
                    let after = self.input.get(self.pos + 1).copied();
                    let is_modulus = matches!(after, Some(c) if c.is_ascii_digit() || c == '(' || c == ' ');
                    if is_modulus {
                        self.advance(); // consume '%'
                        let right = self.power(func_eval)?;
                        if right == 0.0 {
                            return Err(CalcError::DivisionByZero);
                        }
                        left %= right;
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
        Ok(left)
    }

    // power = unary ('^' power)?   (right-associative)
    fn power<F>(&mut self, func_eval: &F) -> Result<f64, CalcError>
    where
        F: Fn(&str, &[f64]) -> Result<f64, CalcError>,
    {
        let base = self.unary(func_eval)?;
        self.skip_whitespace();
        if self.peek() == Some('^') {
            self.advance();
            let exp = self.power(func_eval)?; // right-associative recursion
            Ok(base.powf(exp))
        } else {
            Ok(base)
        }
    }

    // unary = ('-' | '+') unary | postfix
    fn unary<F>(&mut self, func_eval: &F) -> Result<f64, CalcError>
    where
        F: Fn(&str, &[f64]) -> Result<f64, CalcError>,
    {
        self.skip_whitespace();
        match self.peek() {
            Some('-') => {
                self.advance();
                Ok(-self.unary(func_eval)?)
            }
            Some('+') => {
                self.advance();
                self.unary(func_eval)
            }
            _ => self.postfix(func_eval),
        }
    }

    // postfix = primary '%'?
    // '%' as postfix percentage: only when NOT followed by a digit (otherwise modulus)
    fn postfix<F>(&mut self, func_eval: &F) -> Result<f64, CalcError>
    where
        F: Fn(&str, &[f64]) -> Result<f64, CalcError>,
    {
        let val = self.implicit_mul(func_eval)?;
        self.skip_whitespace();
        if self.peek() == Some('%') {
            // Percentage postfix only if NOT followed by digit or '(' (those are modulus)
            let after = self.input.get(self.pos + 1).copied();
            let is_postfix = !matches!(after, Some(c) if c.is_ascii_digit() || c == '(');
            if is_postfix {
                self.advance();
                Ok(val / 100.0)
            } else {
                Ok(val)
            }
        } else {
            Ok(val)
        }
    }

    // implicit_mul handles cases like `2(3)`, `2pi`, `(2)(3)`
    fn implicit_mul<F>(&mut self, func_eval: &F) -> Result<f64, CalcError>
    where
        F: Fn(&str, &[f64]) -> Result<f64, CalcError>,
    {
        let mut val = self.primary(func_eval)?;
        loop {
            self.skip_whitespace();
            // Check if the next token starts something that implies multiplication
            match self.peek() {
                Some('(') => {
                    // implicit mul: number followed by '('
                    val *= self.primary(func_eval)?;
                }
                Some(c) if c == 'π' || self.is_constant_or_ident_start() => {
                    // implicit mul: number followed by constant/identifier
                    val *= self.primary(func_eval)?;
                }
                _ => break,
            }
        }
        Ok(val)
    }

    fn is_constant_or_ident_start(&self) -> bool {
        match self.peek() {
            Some(c) => c.is_ascii_alphabetic() || c == 'π',
            None => false,
        }
    }

    // primary = NUMBER | CONSTANT | '(' expression ')' | FUNC '(' args ')'
    fn primary<F>(&mut self, func_eval: &F) -> Result<f64, CalcError>
    where
        F: Fn(&str, &[f64]) -> Result<f64, CalcError>,
    {
        self.skip_whitespace();

        match self.peek() {
            Some('(') => {
                self.advance();
                let val = self.expression(func_eval)?;
                self.expect(')')?;
                Ok(val)
            }
            Some('π') => {
                self.advance();
                Ok(std::f64::consts::PI)
            }
            Some(c) if c.is_ascii_digit() || c == '.' => self.parse_number(),
            Some(c) if c.is_ascii_alphabetic() => {
                let ident = self.parse_identifier();
                self.skip_whitespace();
                // Check if it's a function call
                if self.peek() == Some('(') {
                    self.advance();
                    let args = self.parse_args(func_eval)?;
                    self.expect(')')?;
                    func_eval(&ident, &args)
                } else {
                    // Try as constant
                    match ident.as_str() {
                        "pi" => Ok(std::f64::consts::PI),
                        "e" => Ok(std::f64::consts::E),
                        _ => Err(CalcError::InvalidExpression(format!(
                            "Unknown identifier: {ident}"
                        ))),
                    }
                }
            }
            Some(c) => Err(CalcError::InvalidExpression(format!(
                "Unexpected character: '{c}'"
            ))),
            None => Err(CalcError::InvalidExpression(
                "Unexpected end of expression".into(),
            )),
        }
    }

    fn parse_number(&mut self) -> Result<f64, CalcError> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '.' {
                self.advance();
            } else {
                break;
            }
        }
        let s: String = self.input[start..self.pos].iter().collect();
        s.parse::<f64>()
            .map_err(|_| CalcError::InvalidExpression(format!("Invalid number: {s}")))
    }

    fn parse_identifier(&mut self) -> String {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        self.input[start..self.pos].iter().collect()
    }

    fn parse_args<F>(&mut self, func_eval: &F) -> Result<Vec<f64>, CalcError>
    where
        F: Fn(&str, &[f64]) -> Result<f64, CalcError>,
    {
        let mut args = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(')') {
            return Ok(args);
        }
        args.push(self.expression(func_eval)?);
        loop {
            self.skip_whitespace();
            if self.peek() == Some(',') {
                self.advance();
                args.push(self.expression(func_eval)?);
            } else {
                break;
            }
        }
        Ok(args)
    }
}
