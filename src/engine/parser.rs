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

        let result = self.bit_or(&func_eval)?;
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

    /// Try to consume the identifier `kw` (case-insensitive). If the upcoming
    /// token is a different identifier (or none), the position is restored and
    /// this returns false — so keywords never clash with function/constant names.
    fn try_keyword(&mut self, kw: &str) -> bool {
        self.skip_whitespace();
        let save = self.pos;
        let start = self.pos;
        while matches!(self.peek(), Some(c) if c.is_ascii_alphanumeric() || c == '_') {
            self.advance();
        }
        let word: String = self.input[start..self.pos].iter().collect();
        if !word.is_empty() && word.eq_ignore_ascii_case(kw) {
            true
        } else {
            self.pos = save;
            false
        }
    }

    /// Peek (without consuming) whether the upcoming identifier is a reserved
    /// bitwise keyword, so `implicit_mul` doesn't swallow `AND`/`OR`/… as an
    /// implicit-multiplication identifier.
    fn upcoming_is_bitwise_keyword(&self) -> bool {
        let mut i = self.pos;
        while i < self.input.len() && (self.input[i].is_ascii_alphanumeric() || self.input[i] == '_')
        {
            i += 1;
        }
        let word: String = self.input[self.pos..i].iter().collect();
        matches!(
            word.to_ascii_uppercase().as_str(),
            "AND" | "OR" | "XOR" | "NOT"
        )
    }

    /// Coerce a value to i64 for bitwise ops, rejecting non-integers and
    /// out-of-range magnitudes.
    fn to_int(v: f64) -> Result<i64, CalcError> {
        if !v.is_finite() || v.fract() != 0.0 || v.abs() >= 9_223_372_036_854_775_808.0 {
            return Err(CalcError::DomainError(
                "bitwise operations require integers".into(),
            ));
        }
        Ok(v as i64)
    }

    // ── Grammar ──────────────────────────────────────────────────
    // Bitwise operators are the loosest (below +/-), C/Python-style:
    //   bit_or < bit_xor < bit_and < shift < expression(+/-) < term(*/) < ...

    // bit_or = bit_xor ('OR' bit_xor)*
    fn bit_or<F>(&mut self, func_eval: &F) -> Result<f64, CalcError>
    where
        F: Fn(&str, &[f64]) -> Result<f64, CalcError>,
    {
        let mut left = self.bit_xor(func_eval)?;
        while self.try_keyword("OR") {
            let right = self.bit_xor(func_eval)?;
            left = (Self::to_int(left)? | Self::to_int(right)?) as f64;
        }
        Ok(left)
    }

    // bit_xor = bit_and ('XOR' bit_and)*
    fn bit_xor<F>(&mut self, func_eval: &F) -> Result<f64, CalcError>
    where
        F: Fn(&str, &[f64]) -> Result<f64, CalcError>,
    {
        let mut left = self.bit_and(func_eval)?;
        while self.try_keyword("XOR") {
            let right = self.bit_and(func_eval)?;
            left = (Self::to_int(left)? ^ Self::to_int(right)?) as f64;
        }
        Ok(left)
    }

    // bit_and = shift ('AND' shift)*
    fn bit_and<F>(&mut self, func_eval: &F) -> Result<f64, CalcError>
    where
        F: Fn(&str, &[f64]) -> Result<f64, CalcError>,
    {
        let mut left = self.shift(func_eval)?;
        while self.try_keyword("AND") {
            let right = self.shift(func_eval)?;
            left = (Self::to_int(left)? & Self::to_int(right)?) as f64;
        }
        Ok(left)
    }

    // shift = expression (('<<' | '>>') expression)*
    fn shift<F>(&mut self, func_eval: &F) -> Result<f64, CalcError>
    where
        F: Fn(&str, &[f64]) -> Result<f64, CalcError>,
    {
        let mut left = self.expression(func_eval)?;
        loop {
            self.skip_whitespace();
            let is_shl = match (self.peek(), self.input.get(self.pos + 1).copied()) {
                (Some('<'), Some('<')) => true,
                (Some('>'), Some('>')) => false,
                _ => break,
            };
            self.advance();
            self.advance();
            let l = Self::to_int(left)?;
            let r = Self::to_int(self.expression(func_eval)?)?;
            if !(0..64).contains(&r) {
                return Err(CalcError::DomainError("shift amount must be 0..63".into()));
            }
            left = (if is_shl { l << r } else { l >> r }) as f64;
        }
        Ok(left)
    }

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
                    let is_modulus =
                        matches!(after, Some(c) if c.is_ascii_digit() || c == '(' || c == ' ');
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
        // Bitwise NOT: unary prefix, binds like unary minus.
        if self.try_keyword("NOT") {
            let v = self.unary(func_eval)?;
            return Ok((!Self::to_int(v)?) as f64);
        }
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
        let mut val = self.implicit_mul(func_eval)?;

        // Factorial: highest-precedence postfix (binds tighter than '^'), and
        // may repeat (e.g. `5!!`). Delegates to the engine's `fact` resolver so
        // the non-negative-integer/overflow guards live in one place.
        loop {
            self.skip_whitespace();
            if self.peek() == Some('!') {
                self.advance();
                val = func_eval("fact", &[val])?;
            } else {
                break;
            }
        }

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
                    // A reserved bitwise keyword (AND/OR/XOR/NOT) is an operator,
                    // not an implicit-multiplication identifier.
                    if self.upcoming_is_bitwise_keyword() {
                        break;
                    }
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
                let val = self.bit_or(func_eval)?;
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
        args.push(self.bit_or(func_eval)?);
        loop {
            self.skip_whitespace();
            if self.peek() == Some(',') {
                self.advance();
                args.push(self.bit_or(func_eval)?);
            } else {
                break;
            }
        }
        Ok(args)
    }
}
