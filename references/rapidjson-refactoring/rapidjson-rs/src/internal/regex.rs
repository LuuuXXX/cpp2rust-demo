//! Regex engine with a Thompson NFA core. This is a staged port of
//! RapidJSON's `GenericRegex` for a subset of patterns. The initial
//! implementation focuses on literals and concatenation, sufficient to
//! satisfy `regextest.cpp`'s `Single` and `Concatenation` cases.

use crate::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Operator {
    ZeroOrOne,
    ZeroOrMore,
    OneOrMore,
    Concatenation,
    Alternation,
    LeftParen,
}

#[derive(Debug, Clone, Copy)]
struct Frag {
    start: usize,
    out: usize,
}

#[derive(Debug, Clone, Copy)]
enum Inst {
    Char(u8),
    Any,
    Class(usize),
    Split(usize, usize),
    Match,
}

#[derive(Debug, Clone)]
struct CharClass {
    ranges: Vec<(u8, u8)>,
    negated: bool,
}

impl CharClass {
    fn matches(&self, ch: u8) -> bool {
        let mut hit = false;
        for &(start, end) in &self.ranges {
            if ch >= start && ch <= end {
                hit = true;
                break;
            }
        }
        if self.negated { !hit } else { hit }
    }
}

/// Helper to build a Thompson NFA program from a pattern.
struct ProgramBuilder<'a> {
    pattern: &'a [u8],
    prog: Vec<Inst>,
    operands: Vec<Frag>,
    operators: Vec<Operator>,
    pos: usize,
    anchor_begin: bool,
    anchor_end: bool,
    classes: Vec<CharClass>,
}

impl<'a> ProgramBuilder<'a> {
    fn new(pattern: &'a str) -> Result<Self, Error> {
        if pattern.is_empty() {
            return Err(Error::Parse {
                message: "empty pattern",
                offset: None,
            });
        }

        Ok(Self {
            pattern: pattern.as_bytes(),
            prog: Vec::new(),
            operands: Vec::new(),
            operators: Vec::new(),
            pos: 0,
            anchor_begin: false,
            anchor_end: false,
            classes: Vec::new(),
        })
    }

    fn finish(&mut self) -> Result<Vec<Inst>, Error> {
        let mut prev_was_atom = false;

        self.pos = 0;
        while self.pos < self.pattern.len() {
            let b = self.pattern[self.pos];
            match b {
                b'^' => {
                    // Beginning anchor: only valid at pattern start.
                    if self.pos != 0 {
                        return Err(Error::Parse {
                            message: "^ anchor only allowed at pattern start",
                            offset: Some(self.pos),
                        });
                    }
                    self.anchor_begin = true;
                }
                b'$' => {
                    // End anchor: mark and otherwise ignore.
                    self.anchor_end = true;
                }
                b'|' => {
                    self.push_operator(Operator::Alternation)?;
                    prev_was_atom = false;
                }
                b'(' => {
                    if prev_was_atom {
                        self.push_operator(Operator::Concatenation)?;
                    }
                    self.operators.push(Operator::LeftParen);
                    prev_was_atom = false;
                }
                b')' => {
                    self.close_parenthesis()?;
                    prev_was_atom = true;
                }
                b'?' => {
                    self.apply_postfix(Operator::ZeroOrOne)?;
                    prev_was_atom = true;
                }
                b'*' => {
                    self.apply_postfix(Operator::ZeroOrMore)?;
                    prev_was_atom = true;
                }
                b'+' => {
                    self.apply_postfix(Operator::OneOrMore)?;
                    prev_was_atom = true;
                }
                b'[' => {
                    if prev_was_atom {
                        self.push_operator(Operator::Concatenation)?;
                    }
                    self.push_char_class()?;
                    prev_was_atom = true;
                }
                b'{' => {
                    self.apply_braced_quantifier()?;
                    prev_was_atom = true;
                }
                b'.' => {
                    if prev_was_atom {
                        self.push_operator(Operator::Concatenation)?;
                    }
                    self.push_atom_any();
                    prev_was_atom = true;
                }
                _ => {
                    if prev_was_atom {
                        self.push_operator(Operator::Concatenation)?;
                    }
                    let c = if b == b'\\' {
                        // escaped literal
                        self.pos += 1;
                        self.parse_escape().ok_or(Error::Parse {
                            message: "invalid escape sequence",
                            offset: Some(self.pos.saturating_sub(1)),
                        })?
                    } else {
                        b
                    };
                    self.push_atom_char(c);
                    prev_was_atom = true;
                }
            }
            self.pos += 1;
        }

        // Drain remaining operators
        while let Some(op) = self.operators.pop() {
            if op == Operator::LeftParen {
                return Err(Error::Parse {
                    message: "unmatched left parenthesis",
                    offset: None,
                });
            }
            self.eval(op)?;
        }

        if self.operands.len() != 1 {
            return Err(Error::Parse {
                message: "incomplete expression",
                offset: None,
            });
        }

        let frag = self.operands.pop().unwrap();
        if frag.out != usize::MAX {
            let match_pc = self.emit(Inst::Match);
            self.patch(frag.out, match_pc);
        } else {
            self.emit(Inst::Match);
        }
        Ok(core::mem::take(&mut self.prog))
    }

    fn push_atom_char(&mut self, c: u8) {
        let start = self.emit(Inst::Char(c));
        self.operands.push(Frag { start, out: usize::MAX });
    }

    fn push_atom_any(&mut self) {
        let start = self.emit(Inst::Any);
        self.operands.push(Frag { start, out: usize::MAX });
    }

    /// Clone the top fragment by copying its instruction slice.
    ///
    /// This is a simplified variant of RapidJSON's CloneTopOperand:
    /// it assumes the last fragment corresponds to a contiguous
    /// sequence of instructions from `start` up to current end.
    fn clone_top_operand(&mut self) -> Result<(), Error> {
        let src = match self.operands.last().copied() {
            Some(frag) => frag,
            None => {
                return Err(Error::Parse {
                    message: "no operand to clone",
                    offset: None,
                })
            }
        };

        let start = src.start;
        let end = self.prog.len();
        if start >= end {
            return Err(Error::Internal);
        }

        let offset = self.prog.len();
        // Clone instructions.
        let slice = self.prog[start..end].to_vec();
        self.prog.extend(slice.iter().copied());

        // Adjust any relative jumps within the cloned range.
        for idx in offset..self.prog.len() {
            if let Inst::Split(x, y) = self.prog[idx] {
                let new_x = if x >= start && x < end {
                    x - start + offset
                } else {
                    x
                };
                let new_y = if y >= start && y < end {
                    y - start + offset
                } else {
                    y
                };
                self.prog[idx] = Inst::Split(new_x, new_y);
            }
        }

        let cloned = Frag {
            start: src.start - start + offset,
            out: if src.out == usize::MAX {
                usize::MAX
            } else {
                src.out - start + offset
            },
        };
        self.operands.push(cloned);
        Ok(())
    }

    fn push_operator(&mut self, op: Operator) -> Result<(), Error> {
        while let Some(&top) = self.operators.last() {
            if top == Operator::LeftParen || top < op {
                break;
            }
            let top = self.operators.pop().unwrap();
            self.eval(top)?;
        }
        self.operators.push(op);
        Ok(())
    }

    fn apply_postfix(&mut self, op: Operator) -> Result<(), Error> {
        if !matches!(op, Operator::ZeroOrOne | Operator::ZeroOrMore | Operator::OneOrMore) {
            return Err(Error::Internal);
        }
        // 量词只作用于最近的 atom，且不跨越括号边界。
        self.eval(op)
    }

    fn apply_braced_quantifier(&mut self) -> Result<(), Error> {
        // We are at '{', parse {n}, {n,} or {n,m} and expand via existing operators.
        let start_pos = self.pos;
        self.pos += 1; // skip '{'
        let n = self.parse_number().ok_or(Error::Parse {
            message: "invalid quantifier",
            offset: Some(start_pos),
        })?;

        let m = if self.peek_byte() == Some(b',') {
            self.pos += 1; // skip ','
            match self.peek_byte() {
                Some(b'}') => None, // {n,}
                _ => {
                    let upper = self.parse_number().ok_or(Error::Parse {
                        message: "invalid quantifier upper bound",
                        offset: Some(start_pos),
                    })?;
                    if upper < n {
                        return Err(Error::Parse {
                            message: "quantifier upper bound < lower bound",
                            offset: Some(start_pos),
                        });
                    }
                    Some(upper)
                }
            }
        } else {
            Some(n) // {n}
        };

        if self.peek_byte() != Some(b'}') {
            return Err(Error::Parse {
                message: "unterminated quantifier",
                offset: Some(start_pos),
            });
        }
        // leave '}' to be consumed by main loop (current char), so do not increment pos here.

        match (n, m) {
            (0, Some(0)) => {
                // a{0} not supported in C++ version either
                return Err(Error::Parse {
                    message: "zero-length quantifier {0} unsupported",
                    offset: Some(start_pos),
                });
            }
            (0, None) => {
                // {0,} -> *
                self.eval_zero_or_more()?;
            }
            (0, Some(upper)) => {
                // {0,m} -> expand as m copies of a? concatenated
                self.eval_zero_or_one()?; // one a?
                for _ in 0..(upper - 1) {
                    self.clone_top_operand()?;
                }
                for _ in 0..(upper - 1) {
                    self.eval_concat()?;
                }
            }
            (n, None) => {
                // {n,} -> n-1 copies + a+ on last
                for _ in 0..(n - 1) {
                    self.clone_top_operand()?;
                }
                self.eval_one_or_more()?; // last a+
                for _ in 0..(n - 1) {
                    self.eval_concat()?;
                }
            }
            (n, Some(upper)) => {
                if upper == n {
                    // {n} -> exactly n copies
                    for _ in 0..(n - 1) {
                        self.clone_top_operand()?;
                    }
                    for _ in 0..(n - 1) {
                        self.eval_concat()?;
                    }
                } else {
                    // {n,m}: n mandatory, up to m with optional tail
                    for _ in 0..(n - 1) {
                        self.clone_top_operand()?;
                    }
                    // one optional copy
                    self.clone_top_operand()?;
                    self.eval_zero_or_one()?;
                    for _ in n..(upper - 1) {
                        self.clone_top_operand()?;
                    }
                    for _ in n..upper {
                        self.eval_concat()?;
                    }
                }
            }
        }

        Ok(())
    }

    fn close_parenthesis(&mut self) -> Result<(), Error> {
        while let Some(&top) = self.operators.last() {
            if top == Operator::LeftParen {
                self.operators.pop();
                return Ok(());
            }
            let op = self.operators.pop().unwrap();
            self.eval(op)?;
        }

        Err(Error::Parse {
            message: "unmatched right parenthesis",
            offset: None,
        })
    }

    fn eval(&mut self, op: Operator) -> Result<(), Error> {
        match op {
            Operator::Concatenation => self.eval_concat(),
            Operator::Alternation => self.eval_alternation(),
            Operator::ZeroOrOne => self.eval_zero_or_one(),
            Operator::ZeroOrMore => self.eval_zero_or_more(),
            Operator::OneOrMore => self.eval_one_or_more(),
            Operator::LeftParen => unreachable!(),
        }
    }

    fn eval_concat(&mut self) -> Result<(), Error> {
        if self.operands.len() < 2 {
            return Err(Error::Parse {
                message: "missing operand for concatenation",
                offset: None,
            });
        }
        let right = self.operands.pop().unwrap();
        let mut left = self.operands.pop().unwrap();

        // Patch left's out to right's start.
        if left.out == usize::MAX {
            left.out = right.start;
        } else {
            self.patch(left.out, right.start);
        }

        let out = if right.out == usize::MAX { left.out } else { right.out };
        self.operands.push(Frag { start: left.start, out });
        Ok(())
    }

    fn eval_alternation(&mut self) -> Result<(), Error> {
        if self.operands.len() < 2 {
            return Err(Error::Parse {
                message: "missing operand for alternation",
                offset: None,
            });
        }
        let right = self.operands.pop().unwrap();
        let left = self.operands.pop().unwrap();

        let split = self.emit(Inst::Split(left.start, right.start));

        // The "out" of an alternation is the combination of both branches.
        let out = match (left.out, right.out) {
            (usize::MAX, usize::MAX) => split,
            (usize::MAX, r) => r,
            (l, usize::MAX) => l,
            (l, r) => {
                self.patch(l, r);
                r
            }
        };

        self.operands.push(Frag { start: split, out });
        Ok(())
    }

    fn eval_zero_or_one(&mut self) -> Result<(), Error> {
        if self.operands.is_empty() {
            return Err(Error::Parse {
                message: "missing operand for ? quantifier",
                offset: None,
            });
        }
        let frag = self.operands.pop().unwrap();
        // Build: epsilon split that can either enter the fragment or skip it.
        let split = self.emit(Inst::Split(frag.start, usize::MAX));
        let out = if frag.out == usize::MAX { split } else { frag.out };
        self.operands.push(Frag { start: split, out });
        Ok(())
    }

    fn eval_zero_or_more(&mut self) -> Result<(), Error> {
        if self.operands.is_empty() {
            return Err(Error::Parse {
                message: "missing operand for * quantifier",
                offset: None,
            });
        }
        let frag = self.operands.pop().unwrap();
        // Build: split that either enters frag or accepts epsilon; tail jumps back.
        let split = self.emit(Inst::Split(frag.start, usize::MAX));
        if frag.out != usize::MAX {
            self.patch(frag.out, split);
        }
        self.operands.push(Frag { start: split, out: split });
        Ok(())
    }

    fn eval_one_or_more(&mut self) -> Result<(), Error> {
        if self.operands.is_empty() {
            return Err(Error::Parse {
                message: "missing operand for + quantifier",
                offset: None,
            });
        }
        let frag = self.operands.pop().unwrap();
        let split = self.emit(Inst::Split(frag.start, usize::MAX));
        if frag.out != usize::MAX {
            self.patch(frag.out, split);
        }
        self.operands.push(Frag { start: frag.start, out: split });
        Ok(())
    }

    fn parse_escape(&mut self) -> Option<u8> {
        let b = *self.pattern.get(self.pos)?;
        self.pos += 1;
        Some(match b {
            b'^' | b'$' | b'|' | b'(' | b')' | b'?' | b'*' | b'+' | b'.'
            | b'[' | b']' | b'{' | b'}' | b'\\' => b,
            b'f' => 0x0C, // form feed
            b'n' => b'\n',
            b'r' => b'\r',
            b't' => b'\t',
            b'v' => 0x0B,
            b'b' => 0x08,
            _ => return None,
        })
    }

    fn parse_number(&mut self) -> Option<u32> {
        let mut value: u32 = 0;
        let mut saw_digit = false;
        while let Some(b) = self.peek_byte() {
            if !(b'0'..=b'9').contains(&b) {
                break;
            }
            saw_digit = true;
            let d = (b - b'0') as u32;
            // very conservative overflow guard, not critical for our usage
            if let Some(v) = value.checked_mul(10).and_then(|v| v.checked_add(d)) {
                value = v;
            } else {
                return None;
            }
            self.pos += 1;
        }
        if saw_digit { Some(value) } else { None }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.pattern.get(self.pos).copied()
    }

    fn push_char_class(&mut self) -> Result<(), Error> {
        // We are at '[', parse until closing ']'.
        let start_pos = self.pos;
        self.pos += 1; // skip '['
        let mut negated = false;
        let mut ranges = Vec::<(u8, u8)>::new();

        if self.peek_byte() == Some(b'^') {
            negated = true;
            self.pos += 1;
        }

        let mut first = true;
        let mut pending_range_start: Option<u8> = None;

        while let Some(b) = self.peek_byte() {
            if b == b']' && !first {
                // end of class
                self.pos += 1; // consume ']'
                break;
            }
            first = false;

            let ch = if b == b'\\' {
                self.pos += 1;
                self.parse_escape().ok_or(Error::Parse {
                    message: "invalid escape in character class",
                    offset: Some(self.pos.saturating_sub(1)),
                })?
            } else {
                self.pos += 1;
                b
            };

            if ch == b'-' && pending_range_start.is_some() && self.peek_byte() != Some(b']') {
                // range syntax: a-b
                let next = if self.peek_byte() == Some(b'\\') {
                    self.pos += 1;
                    self.parse_escape().ok_or(Error::Parse {
                        message: "invalid escape in character class range",
                        offset: Some(self.pos.saturating_sub(1)),
                    })?
                } else {
                    let nb = self.peek_byte().ok_or(Error::Parse {
                        message: "unterminated character class range",
                        offset: Some(self.pos),
                    })?;
                    self.pos += 1;
                    nb
                };
                let start = pending_range_start.take().unwrap();
                let (lo, hi) = if start <= next { (start, next) } else { (next, start) };
                ranges.push((lo, hi));
            } else {
                if let Some(start) = pending_range_start.take() {
                    ranges.push((start, start));
                }
                pending_range_start = Some(ch);
            }
        }

        if pending_range_start.is_some() {
            let start = pending_range_start.unwrap();
            ranges.push((start, start));
        }

        if ranges.is_empty() {
            return Err(Error::Parse {
                message: "empty character class",
                offset: Some(start_pos),
            });
        }

        let class_index = self.classes.len();
        self.classes.push(CharClass { ranges, negated });
        let start_pc = self.emit(Inst::Class(class_index));
        self.operands.push(Frag {
            start: start_pc,
            out: usize::MAX,
        });
        Ok(())
    }

    fn emit(&mut self, inst: Inst) -> usize {
        let pc = self.prog.len();
        self.prog.push(inst);
        pc
    }

    fn patch(&mut self, from: usize, to: usize) {
        if let Some(inst) = self.prog.get_mut(from) {
            if let Inst::Split(ref mut x, ref mut y) = *inst {
                if *x == usize::MAX {
                    *x = to;
                } else if *y == usize::MAX {
                    *y = to;
                }
            }
        }
    }
}

/// Compiled regular expression program.
pub struct Regex {
    prog: Vec<Inst>,
    anchor_begin: bool,
    anchor_end: bool,
    classes: Vec<CharClass>,
}

/// NFA-based regex search over a compiled `Regex` program.
pub struct RegexSearch<'a> {
    regex: &'a Regex,
}

impl Regex {
    /// Compiles a new regex from the given pattern. The initial
    /// implementation only supports literals and `.`, without
    /// alternation or grouping.
    pub fn new(pattern: &str) -> Result<Self, Error> {
        if pattern.is_empty() {
            return Err(Error::Parse {
                message: "empty pattern",
                offset: None,
            });
        }

        let mut builder = ProgramBuilder::new(pattern)?;
        let prog = builder.finish()?;
        Ok(Self {
            prog,
            anchor_begin: builder.anchor_begin,
            anchor_end: builder.anchor_end,
            classes: builder.classes,
        })
    }

    pub fn search<'a>(&'a self) -> RegexSearch<'a> {
        RegexSearch { regex: self }
    }
}

impl<'a> RegexSearch<'a> {
    pub fn is_match(&self, input: &str) -> bool {
        // is_match 等价于带锚点的匹配：^pattern$。
        self.match_full(input.as_bytes())
    }

    /// Search for the pattern as a substring, honoring ^ / $ anchors.
    pub fn search(&self, input: &str) -> bool {
        let bytes = input.as_bytes();
        // Anchor handling:
        // - ^ and $ both set: full match only.
        // - ^ only: must match at beginning.
        // - $ only: must match at end.
        // - neither: classic substring search.
        if self.regex.anchor_begin && self.regex.anchor_end {
            return self.match_full(bytes);
        }

        if self.regex.anchor_begin {
            // Only match at the beginning.
            return self.match_prefix(bytes);
        }

        if self.regex.anchor_end {
            // Only match at the end.
            return self.match_suffix(bytes);
        }

        // Unanchored: scan all possible starting positions.
        if bytes.is_empty() {
            return self.match_prefix(&[]);
        }

        for start in 0..bytes.len() {
            if self.match_prefix(&bytes[start..]) {
                return true;
            }
        }
        false
    }

    fn match_full(&self, text: &[u8]) -> bool {
        // Full-match NFA: run from start, require end at Match with
        // text fully consumed.
        let mut current = Vec::new();
        let mut next = Vec::new();
        self.add_state(0, &mut current);

        for &ch in text {
            next.clear();
            for &pc in &current {
                match self.regex.prog[pc] {
                    Inst::Char(c) if c == ch => self.add_state(pc + 1, &mut next),
                    Inst::Any => self.add_state(pc + 1, &mut next),
                    Inst::Class(idx) => {
                        if self.regex.classes[idx].matches(ch) {
                            self.add_state(pc + 1, &mut next)
                        }
                    }
                    _ => {}
                }
            }
            core::mem::swap(&mut current, &mut next);
        }

        // Check if any of the current states is a Match.
        current
            .iter()
            .any(|&pc| matches!(self.regex.prog[pc], Inst::Match))
    }

    /// Match pattern starting at the beginning of `text`, without requiring
    /// the whole `text` to be consumed.
    fn match_prefix(&self, text: &[u8]) -> bool {
        let mut current = Vec::new();
        let mut next = Vec::new();
        self.add_state(0, &mut current);

        for &ch in text {
            next.clear();
            for &pc in &current {
                match self.regex.prog[pc] {
                    Inst::Char(c) if c == ch => self.add_state(pc + 1, &mut next),
                    Inst::Any => self.add_state(pc + 1, &mut next),
                    Inst::Class(idx) => {
                        if self.regex.classes[idx].matches(ch) {
                            self.add_state(pc + 1, &mut next)
                        }
                    }
                    _ => {}
                }
            }
            core::mem::swap(&mut current, &mut next);

            if current
                .iter()
                .any(|&pc| matches!(self.regex.prog[pc], Inst::Match))
            {
                return true;
            }
        }

        current
            .iter()
            .any(|&pc| matches!(self.regex.prog[pc], Inst::Match))
    }

    /// Match pattern so that it ends at the end of `text`.
    fn match_suffix(&self, text: &[u8]) -> bool {
        // For simplicity, scan all possible prefixes whose suffix aligns with
        // the end. This is O(n^2) but acceptable for the simplified engine.
        for start in 0..=text.len() {
            let slice = &text[start..];
            if !slice.is_empty() && self.match_full(slice) {
                return true;
            }
        }
        false
    }

    fn add_state(&self, mut pc: usize, states: &mut Vec<usize>) {
        // Follow all epsilon transitions (Split/Jmp).
        loop {
            if pc >= self.regex.prog.len() {
                return;
            }
            match self.regex.prog[pc] {
                Inst::Split(x, y) => {
                    // Explore both epsilon branches.
                    self.add_state(x, states);
                    pc = y;
                }
                _ => {
                    if !states.contains(&pc) {
                        states.push(pc);
                    }
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Regex;

    #[test]
    fn should_match_single_literal_when_regex_single() {
        let re = Regex::new("a").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("a"));
        assert!(!rs.is_match(""));
        assert!(!rs.is_match("b"));
    }

    #[test]
    fn should_match_concatenation_when_regex_concatenation() {
        let re = Regex::new("abc").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("abc"));
        assert!(!rs.is_match(""));
        assert!(!rs.is_match("a"));
        assert!(!rs.is_match("b"));
        assert!(!rs.is_match("ab"));
        assert!(!rs.is_match("abcd"));
    }

    #[test]
    fn should_search_unanchored_pattern_as_substring() {
        let re = Regex::new("abc").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.search("abc"));
        assert!(rs.search("_abc"));
        assert!(rs.search("abc_"));
        assert!(rs.search("_abc_"));
        assert!(rs.search("__abc__"));
        assert!(rs.search("abcabc"));
        assert!(!rs.search("a"));
        assert!(!rs.search("ab"));
        assert!(!rs.search("bc"));
        assert!(!rs.search("cba"));
    }

    #[test]
    fn should_respect_begin_anchor_when_search() {
        let re = Regex::new("^abc").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.search("abc"));
        assert!(rs.search("abc_"));
        assert!(rs.search("abcabc"));
        assert!(!rs.search("_abc"));
        assert!(!rs.search("_abc_"));
        assert!(!rs.search("a"));
        assert!(!rs.search("ab"));
        assert!(!rs.search("bc"));
        assert!(!rs.search("cba"));
    }

    #[test]
    fn should_respect_end_anchor_when_search() {
        let re = Regex::new("abc$").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.search("abc"));
        assert!(rs.search("_abc"));
        assert!(rs.search("abcabc"));
        assert!(!rs.search("abc_"));
        assert!(!rs.search("_abc_"));
        assert!(!rs.search("a"));
        assert!(!rs.search("ab"));
        assert!(!rs.search("bc"));
        assert!(!rs.search("cba"));
    }

    #[test]
    fn should_respect_both_anchors_when_search() {
        let re = Regex::new("^abc$").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.search("abc"));
        assert!(!rs.search(""));
        assert!(!rs.search("a"));
        assert!(!rs.search("b"));
        assert!(!rs.search("ab"));
        assert!(!rs.search("abcd"));
    }

    #[test]
    fn should_support_simple_character_class() {
        let re = Regex::new("[abc]").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("a"));
        assert!(rs.is_match("b"));
        assert!(rs.is_match("c"));
        assert!(!rs.is_match(""));
        assert!(!rs.is_match("`"));
        assert!(!rs.is_match("d"));
        assert!(!rs.is_match("aa"));
    }

    #[test]
    fn should_support_negated_character_class() {
        let re = Regex::new("[^abc]").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("`"));
        assert!(rs.is_match("d"));
        assert!(!rs.is_match("a"));
        assert!(!rs.is_match("b"));
        assert!(!rs.is_match("c"));
        assert!(!rs.is_match(""));
        assert!(!rs.is_match("aa"));
    }

    #[test]
    fn should_support_character_range() {
        let re = Regex::new("[a-c]").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("a"));
        assert!(rs.is_match("b"));
        assert!(rs.is_match("c"));
        assert!(!rs.is_match(""));
        assert!(!rs.is_match("`"));
        assert!(!rs.is_match("d"));
        assert!(!rs.is_match("aa"));
    }

    #[test]
    fn should_support_negated_character_range() {
        let re = Regex::new("[^a-c]").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("`"));
        assert!(rs.is_match("d"));
        assert!(!rs.is_match("a"));
        assert!(!rs.is_match("b"));
        assert!(!rs.is_match("c"));
        assert!(!rs.is_match(""));
        assert!(!rs.is_match("aa"));
    }

    #[test]
    fn should_support_single_dash_in_character_class() {
        let re = Regex::new("[-]").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("-"));
        assert!(!rs.is_match(""));
        assert!(!rs.is_match("a"));
    }

    #[test]
    fn should_support_dash_at_character_class_end() {
        let re = Regex::new("[a-]").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("a"));
        assert!(rs.is_match("-"));
        assert!(!rs.is_match(""));
        assert!(!rs.is_match("`"));
        assert!(!rs.is_match("b"));
    }

    #[test]
    fn should_support_dash_at_character_class_begin() {
        let re = Regex::new("[-a]").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("a"));
        assert!(rs.is_match("-"));
        assert!(!rs.is_match(""));
        assert!(!rs.is_match("`"));
        assert!(!rs.is_match("b"));
    }

    #[test]
    fn should_support_escape_sequence_for_special_chars() {
        // Mirror Escape test: pattern represents a sequence of escaped specials and control chars.
        let pattern = "\\^\\$\\|\\(\\)\\?\\*\\+\\.\\[\\]\\{\\}\\\\\\f\\n\\r\\t\\v[\\b][\\[][\\]]";
        let re = Regex::new(pattern).expect("compile should succeed");
        let rs = re.search();

        let target = "^$|()?*+.[]{}\\\n\r\t[]";
        assert!(rs.is_match(target));
        assert!(!rs.is_match(pattern));
    }

    #[test]
    fn should_treat_unicode_bytes_as_literal_sequences() {
        // Simplified byte-based Unicode behaviour test: use raw UTF-8 bytes.
        let euro = ""; // placeholder single-byte non-ASCII.
        let pattern = format!("a{}+b", euro);
        let re = Regex::new(&pattern).expect("compile should succeed");
        let rs = re.search();

        let s1 = format!("a{}b", euro);
        let s2 = format!("a{}{}b", euro, euro);
        assert!(rs.is_match(&s1));
        assert!(rs.is_match(&s2));
    }

    #[test]
    fn should_match_either_side_when_alternation() {
        let re = Regex::new("abab|abbb").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("abab"));
        assert!(rs.is_match("abbb"));
        assert!(!rs.is_match(""));
        assert!(!rs.is_match("ab"));
        assert!(!rs.is_match("ababa"));
        assert!(!rs.is_match("abb"));
        assert!(!rs.is_match("abbbb"));
    }

    #[test]
    fn should_match_any_of_three_when_alternation_chain() {
        let re = Regex::new("a|b|c").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("a"));
        assert!(rs.is_match("b"));
        assert!(rs.is_match("c"));
        assert!(!rs.is_match(""));
        assert!(!rs.is_match("aa"));
        assert!(!rs.is_match("ab"));
    }

    #[test]
    fn should_respect_grouping_prefix_when_parenthesis_left() {
        let re = Regex::new("(ab)c").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("abc"));
        assert!(!rs.is_match(""));
        assert!(!rs.is_match("a"));
        assert!(!rs.is_match("b"));
        assert!(!rs.is_match("ab"));
        assert!(!rs.is_match("abcd"));
    }

    #[test]
    fn should_respect_grouping_suffix_when_parenthesis_right() {
        let re = Regex::new("a(bc)").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("abc"));
        assert!(!rs.is_match(""));
        assert!(!rs.is_match("a"));
        assert!(!rs.is_match("b"));
        assert!(!rs.is_match("ab"));
        assert!(!rs.is_match("abcd"));
    }

    #[test]
    fn should_support_alternation_inside_groups() {
        let re = Regex::new("(a|b)(c|d)").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("ac"));
        assert!(rs.is_match("ad"));
        assert!(rs.is_match("bc"));
        assert!(rs.is_match("bd"));
        assert!(!rs.is_match(""));
        assert!(!rs.is_match("ab"));
        assert!(!rs.is_match("cd"));
    }

    #[test]
    fn should_support_zero_or_one_quantifier_simple() {
        let re = Regex::new("a?").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match(""));
        assert!(rs.is_match("a"));
        assert!(!rs.is_match("aa"));
    }

    #[test]
    fn should_support_zero_or_one_quantifier_with_suffix() {
        let re = Regex::new("a?b").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("b"));
        assert!(rs.is_match("ab"));
        assert!(!rs.is_match("a"));
        assert!(!rs.is_match("aa"));
        assert!(!rs.is_match("bb"));
        assert!(!rs.is_match("ba"));
    }

    #[test]
    fn should_support_zero_or_one_quantifier_trailing() {
        let re = Regex::new("ab?").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("a"));
        assert!(rs.is_match("ab"));
        assert!(!rs.is_match("b"));
        assert!(!rs.is_match("aa"));
        assert!(!rs.is_match("bb"));
        assert!(!rs.is_match("ba"));
    }

    #[test]
    fn should_support_zero_or_one_quantifier_twice() {
        let re = Regex::new("a?b?").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match(""));
        assert!(rs.is_match("a"));
        assert!(rs.is_match("b"));
        assert!(rs.is_match("ab"));
        assert!(!rs.is_match("aa"));
        assert!(!rs.is_match("bb"));
        assert!(!rs.is_match("ba"));
        assert!(!rs.is_match("abc"));
    }

    #[test]
    fn should_support_zero_or_one_quantifier_on_group() {
        let re = Regex::new("a(ab)?b").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("ab"));
        assert!(rs.is_match("aabb"));
        assert!(!rs.is_match("aab"));
        assert!(!rs.is_match("abb"));
    }

    #[test]
    fn should_support_zero_or_more_quantifier_simple() {
        let re = Regex::new("a*").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match(""));
        assert!(rs.is_match("a"));
        assert!(rs.is_match("aa"));
        assert!(!rs.is_match("b"));
        assert!(!rs.is_match("ab"));
    }

    #[test]
    fn should_support_zero_or_more_quantifier_with_suffix() {
        let re = Regex::new("a*b").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("b"));
        assert!(rs.is_match("ab"));
        assert!(rs.is_match("aab"));
        assert!(!rs.is_match(""));
        assert!(!rs.is_match("bb"));
    }

    #[test]
    fn should_support_zero_or_more_quantifier_two_symbols() {
        let re = Regex::new("a*b*").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match(""));
        assert!(rs.is_match("a"));
        assert!(rs.is_match("aa"));
        assert!(rs.is_match("b"));
        assert!(rs.is_match("bb"));
        assert!(rs.is_match("ab"));
        assert!(rs.is_match("aabb"));
        assert!(!rs.is_match("ba"));
    }

    #[test]
    fn should_support_zero_or_more_quantifier_on_group() {
        let re = Regex::new("a(ab)*b").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("ab"));
        assert!(rs.is_match("aabb"));
        assert!(rs.is_match("aababb"));
        assert!(!rs.is_match(""));
        assert!(!rs.is_match("aa"));
    }

    #[test]
    fn should_support_one_or_more_quantifier_simple() {
        let re = Regex::new("a+").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("a"));
        assert!(rs.is_match("aa"));
        assert!(!rs.is_match(""));
        assert!(!rs.is_match("b"));
        assert!(!rs.is_match("ab"));
    }

    #[test]
    fn should_support_one_or_more_quantifier_with_suffix() {
        let re = Regex::new("a+b").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("ab"));
        assert!(rs.is_match("aab"));
        assert!(!rs.is_match(""));
        assert!(!rs.is_match("b"));
    }

    #[test]
    fn should_support_one_or_more_quantifier_two_symbols() {
        let re = Regex::new("a+b+").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("ab"));
        assert!(rs.is_match("aab"));
        assert!(rs.is_match("abb"));
        assert!(rs.is_match("aabb"));
        assert!(!rs.is_match(""));
        assert!(!rs.is_match("b"));
        assert!(!rs.is_match("ba"));
    }

    #[test]
    fn should_support_one_or_more_quantifier_on_group() {
        let re = Regex::new("a(ab)+b").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("aabb"));
        assert!(rs.is_match("aababb"));
        assert!(!rs.is_match(""));
        assert!(!rs.is_match("ab"));
    }

    #[test]
    fn should_support_exact_quantifier_on_literal() {
        let re = Regex::new("ab{3}c").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("abbbc"));
        assert!(!rs.is_match("ac"));
        assert!(!rs.is_match("abc"));
        assert!(!rs.is_match("abbc"));
        assert!(!rs.is_match("abbbbc"));
    }

    #[test]
    fn should_support_exact_quantifier_on_group() {
        let re = Regex::new("a(bc){3}d").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("abcbcbcd"));
        assert!(!rs.is_match("ad"));
        assert!(!rs.is_match("abcd"));
        assert!(!rs.is_match("abcbcd"));
        assert!(!rs.is_match("abcbcbcbcd"));
    }

    #[test]
    fn should_support_exact_quantifier_on_group_with_alternation() {
        let re = Regex::new("a(b|c){3}d").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("abbbd"));
        assert!(rs.is_match("acccd"));
        assert!(rs.is_match("abcbd"));
        assert!(!rs.is_match("ad"));
        assert!(!rs.is_match("abbd"));
        assert!(!rs.is_match("accccd"));
        assert!(!rs.is_match("abbbbd"));
    }

    #[test]
    fn should_support_min_quantifier_on_literal() {
        let re = Regex::new("ab{3,}c").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("abbbc"));
        assert!(rs.is_match("abbbbc"));
        assert!(rs.is_match("abbbbbc"));
        assert!(!rs.is_match("ac"));
        assert!(!rs.is_match("abc"));
        assert!(!rs.is_match("abbc"));
    }

    #[test]
    fn should_support_min_quantifier_on_group() {
        let re = Regex::new("a(bc){3,}d").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("abcbcbcd"));
        assert!(rs.is_match("abcbcbcbcd"));
        assert!(!rs.is_match("ad"));
        assert!(!rs.is_match("abcd"));
        assert!(!rs.is_match("abcbcd"));
    }

    #[test]
    fn should_support_min_quantifier_on_group_with_alternation() {
        let re = Regex::new("a(b|c){3,}d").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("abbbd"));
        assert!(rs.is_match("acccd"));
        assert!(rs.is_match("abcbd"));
        assert!(rs.is_match("accccd"));
        assert!(rs.is_match("abbbbd"));
        assert!(!rs.is_match("ad"));
        assert!(!rs.is_match("abbd"));
    }

    #[test]
    fn should_support_min_max_quantifier_on_literal() {
        let re = Regex::new("ab{3,5}c").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("abbbc"));
        assert!(rs.is_match("abbbbc"));
        assert!(rs.is_match("abbbbbc"));
        assert!(!rs.is_match("ac"));
        assert!(!rs.is_match("abc"));
        assert!(!rs.is_match("abbc"));
        assert!(!rs.is_match("abbbbbbc"));
    }

    #[test]
    fn should_support_min_max_quantifier_on_group() {
        let re = Regex::new("a(bc){3,5}d").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("abcbcbcd"));
        assert!(rs.is_match("abcbcbcbcd"));
        assert!(rs.is_match("abcbcbcbcbcd"));
        assert!(!rs.is_match("ad"));
        assert!(!rs.is_match("abcd"));
        assert!(!rs.is_match("abcbcd"));
        assert!(!rs.is_match("abcbcbcbcbcbcd"));
    }

    #[test]
    fn should_support_min_max_quantifier_on_group_with_alternation() {
        let re = Regex::new("a(b|c){3,5}d").expect("compile should succeed");
        let rs = re.search();
        assert!(rs.is_match("abbbd"));
        assert!(rs.is_match("acccd"));
        assert!(rs.is_match("abcbd"));
        assert!(rs.is_match("accccd"));
        assert!(rs.is_match("abbbbd"));
        assert!(rs.is_match("acccccd"));
        assert!(rs.is_match("abbbbbd"));
        assert!(!rs.is_match("ad"));
        assert!(!rs.is_match("abbd"));
        assert!(!rs.is_match("accccccd"));
        assert!(!rs.is_match("abbbbbbd"));
    }
}
