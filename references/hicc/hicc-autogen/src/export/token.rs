
pub(super) fn first_group(input: &str, x: (u8, u8)) -> (Option<usize>, Option<usize>) {
    let input = input.as_bytes();
    let mut deep = 0;
    let mut beg = None;
    for (n, b) in input.iter().enumerate() {
        if *b == x.0 {
            if beg.is_none() {
                beg = Some(n);
            }
            deep += 1;
        } else if *b == x.1 {
            if deep == 0 {
                return (None, Some(n));
            }
            deep -= 1;
            if deep == 0 {
                return (beg, Some(n));
            }
        }
    }
    (beg, None)
}

pub(super) fn last_group(input: &str, x: (u8, u8)) -> (Option<usize>, Option<usize>) {
    let input = input.as_bytes();
    let mut deep = 0;
    let mut end = None;
    for (n, b) in input.iter().enumerate().rev() {
        if *b == x.1 {
            if end.is_none() {
                end = Some(n);
            }
            deep += 1;
        } else if *b == x.0 {
            if deep == 0 {
                return (Some(n), None);
            }
            deep -= 1;
            if deep == 0 {
                return (Some(n), end);
            }
        }
    }
    (None, end)
}

pub(super) fn is_ident(b: u8) -> bool {
    matches!(b, b'_' | b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'@' | b'$')
}

pub(super) fn first_ident(input: &str) -> (usize, usize) {
    let left = input.trim_start();
    let beg = input.len() - left.len();
    for (n, b) in left.as_bytes().iter().enumerate() {
        if !is_ident(*b) {
            return (beg, beg + n);
        }
    }
    (beg, beg + left.len())
}

pub(super) fn last_ident(input: &str) -> (usize, usize) {
    let input = input.trim_end();
    for (n, b) in input.as_bytes().iter().enumerate().rev() {
        if !is_ident(*b) {
            return (n + 1, input.len());
        }
    }
    (0, input.len())
}

pub(super) struct TokenStream<'a> {
    pub input: &'a str,
    pub offset: usize,
    pub end: usize,
}

impl<'a> Clone for TokenStream<'a> {
    fn clone(&self) -> Self {
        Self {
            input: self.input,
            offset: self.offset,
            end: self.end,
        }
    }
}

#[allow(dead_code)]
impl<'a> TokenStream<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            offset: 0,
            end: input.len(),
        }
    }

    pub fn set_range(&mut self, offset: usize, end: usize) {
        self.offset = offset;
        self.end = end;
    }

    pub fn next_token(&mut self) -> Option<&'a str> {
        let token = self.next();
        self.get_token(token)
    }

    pub fn prev_token(&mut self) -> Option<&'a str> {
        let token = self.next_back();
        self.get_token(token)
    }

    pub fn next_path(&mut self) -> Option<&'a str> {
        self.next_path_range().map(|(x1, x2)| &self.input[x1..x2])
    }

    pub fn next_path_range(&mut self) -> Option<(usize, usize)> {
        let mut stream = self.clone();
        let mut x1 = 0;
        let mut prev = None;
        for current in &mut stream {
            match (&prev, &current) {
                (Some(TokenTree::Ident(_)), TokenTree::Group(_, b'<')) => prev = Some(current),
                (Some(TokenTree::Ident(Range(_, x1))), TokenTree::Punct(Range(x2, _), b':'))
                    if x2 == x1 =>
                {
                    prev = Some(current)
                }
                (
                    Some(TokenTree::Group(Range(_, x1), b'<')),
                    TokenTree::Punct(Range(x2, _), b':'),
                ) if x2 == x1 => prev = Some(current),
                (Some(TokenTree::Punct(Range(_, x1), b':')), TokenTree::Ident(Range(x2, _)))
                    if x2 == x1 =>
                {
                    prev = Some(current)
                }
                (
                    Some(TokenTree::Punct(Range(_, x1), b':')),
                    TokenTree::Punct(Range(x2, _), b':'),
                ) if x2 == x1 => prev = Some(current),
                (None, TokenTree::Punct(Range(x, _), b':')) => {
                    x1 = *x;
                    prev = Some(current);
                }
                (None, TokenTree::Ident(Range(x, _))) => {
                    x1 = *x;
                    prev = Some(current);
                }
                _ => break,
            }
        }
        let x2 = match prev {
            Some(TokenTree::Group(Range(_, x2), b'<')) => x2 + 1,
            Some(TokenTree::Ident(Range(_, x2))) => x2,
            _ => return None,
        };
        self.offset = x2;
        Some((x1, x2))
    }

    pub fn prev_path(&mut self) -> Option<&'a str> {
        self.prev_path_range().map(|(x1, x2)| &self.input[x1..x2])
    }

    pub fn prev_path_range(&mut self) -> Option<(usize, usize)> {
        let mut stream = self.clone();
        let mut x2 = 0;
        let mut prev = None;
        while let Some(current) = stream.next_back() {
            match (&current, &prev) {
                (TokenTree::Ident(_), Some(TokenTree::Group(_, b'<'))) => prev = Some(current),
                (TokenTree::Ident(Range(_, x1)), Some(TokenTree::Punct(Range(x2, _), b':')))
                    if x2 == x1 =>
                {
                    prev = Some(current)
                }
                (
                    TokenTree::Group(Range(_, x1), b'<'),
                    Some(TokenTree::Punct(Range(x2, _), b':')),
                ) if x2 == x1 => prev = Some(current),
                (
                    TokenTree::Punct(Range(_, x1), b':'),
                    Some(TokenTree::Punct(Range(x2, _), b':')),
                ) if x2 == x1 => prev = Some(current),
                (TokenTree::Punct(Range(_, x1), b':'), Some(TokenTree::Ident(Range(x2, _))))
                    if x2 == x1 =>
                {
                    prev = Some(current)
                }
                (TokenTree::Group(Range(_, x), b'<'), None) => {
                    x2 = *x + 1;
                    prev = Some(current);
                }
                (TokenTree::Ident(Range(_, x)), None) => {
                    x2 = *x;
                    prev = Some(current);
                }
                _ => break,
            }
        }
        let x1 = match prev {
            Some(TokenTree::Punct(Range(x1, _), b':')) => x1,
            Some(TokenTree::Ident(Range(x1, _))) => x1,
            _ => return None,
        };
        self.end = x1;
        Some((x1, x2))
    }

    pub fn get_token(&self, token: Option<TokenTree>) -> Option<&'a str> {
        token.map(|token| {
            let (beg, end) = token.range();
            &self.input[beg..end]
        })
    }
}

pub(super) struct Range(pub usize, pub usize);

pub(super) enum TokenTree {
    Group(Range, u8),
    Ident(Range),
    Punct(Range, u8),
}

impl TokenTree {
    pub fn range(&self) -> (usize, usize) {
        match self {
            Self::Group(Range(x1, x2), _) => (*x1, *x2 + 1),
            Self::Ident(Range(x1, x2)) => (*x1, *x2),
            Self::Punct(Range(x1, x2), _) => (*x1, *x2),
        }
    }
}

impl Iterator for TokenStream<'_> {
    type Item = TokenTree;
    fn next(&mut self) -> Option<Self::Item> {
        let input = self.input[self.offset..].trim_start();
        let offset = self.input.len() - input.len();
        if offset >= self.end {
            return None;
        }
        let input = &input[..self.end - offset];
        let (open, close) = match input.as_bytes().first() {
            Some(&b'(') => (b'(', b')'),
            Some(&b'[') => (b'[', b']'),
            Some(&b'<') => (b'<', b'>'),
            Some(&b'{') => (b'{', b'}'),
            Some(c) if is_ident(*c) => {
                let (mut beg, mut end) = first_ident(input);
                beg += offset;
                end += offset;
                self.offset = end;
                return Some(TokenTree::Ident(Range(beg, end)));
            }
            Some(c) => {
                self.offset = offset + 1;
                return Some(TokenTree::Punct(Range(self.offset - 1, self.offset), *c));
            }
            None => return None,
        };
        let (Some(mut beg), Some(mut end)) = first_group(input, (open, close)) else {
            return None;
        };
        beg += offset;
        end += offset;
        self.offset = end + 1;
        Some(TokenTree::Group(Range(beg, end), open))
    }
}

impl DoubleEndedIterator for TokenStream<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let input = self.input[..self.end].trim_end();
        let end = input.len();
        if self.offset >= end {
            return None;
        }
        let (open, close) = match input.as_bytes().last() {
            Some(&b')') => (b'(', b')'),
            Some(&b']') => (b'[', b']'),
            Some(&b'>') => (b'<', b'>'),
            Some(&b'}') => (b'{', b'}'),
            Some(c) if is_ident(*c) => {
                let (beg, end) = last_ident(input);
                self.end = beg;
                return Some(TokenTree::Ident(Range(beg, end)));
            }
            Some(c) => {
                self.end = end - 1;
                return Some(TokenTree::Punct(Range(self.end, self.end + 1), *c));
            }
            None => return None,
        };
        let (Some(beg), Some(end)) = last_group(input, (open, close)) else {
            return None;
        };
        self.end = beg;
        Some(TokenTree::Group(Range(beg, end), open))
    }
}

pub(super) fn parse_func_type(input: &str) -> (usize, usize, Option<(usize, usize)>) {
    let (mut beg, mut end) = (0, input.len());
    let mut stream = TokenStream::new(input);
    'outer: while let Some(token) = stream.next() {
        let TokenTree::Group(Range(x1, x2), b'(') = token else {
            continue;
        };
        while let Some(token) = stream.next() {
            if matches!(token, TokenTree::Group(_, b'(')) {
                beg = x1 + 1;
                end = x2;
                //函数返回函数指针类型.
                stream.set_range(beg, end);
                continue 'outer;
            }
        }
        return (beg, end, Some((x1, x2)));
    }
    (beg, end, None)
}

pub(super) fn parse_function(input: &str) -> Option<(usize, usize, usize, usize, usize)> {
    let (_, x5, Some((x3, x4))) = parse_func_type(input) else {
        return None;
    };
    let mut stream = TokenStream::new(&input[..x3]);
    let (x1, x2) = stream.prev_path_range()?;
    Some((x1, x2, x3, x4, x5))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test0() {
        let input = "void std::fooo(int, const char*)";
        let mut stream = TokenStream::new(input);
        assert_eq!(stream.next_token(), Some("void"));
        assert_eq!(stream.next_token(), Some("std"));
        assert_eq!(stream.next_token(), Some(":"));
        assert_eq!(stream.next_token(), Some(":"));
        assert_eq!(stream.next_token(), Some("fooo"));
        assert_eq!(stream.next_token(), Some("(int, const char*)"));
        assert_eq!(stream.next_token(), None);
        let mut stream = TokenStream::new(input);
        assert_eq!(stream.prev_token(), Some("(int, const char*)"));
        assert_eq!(stream.prev_token(), Some("fooo"));
        assert_eq!(stream.prev_token(), Some(":"));
        assert_eq!(stream.prev_token(), Some(":"));
        assert_eq!(stream.prev_token(), Some("std"));
        assert_eq!(stream.prev_token(), Some("void"));
        assert_eq!(stream.prev_token(), None);

        let input = "int (* foo(int))(int)";
        let mut stream = TokenStream::new(input);
        assert_eq!(stream.next_token(), Some("int"));
        assert_eq!(stream.next_token(), Some("(* foo(int))"));
        assert_eq!(stream.next_token(), Some("(int)"));
        assert_eq!(stream.next_token(), None);
        let mut stream = TokenStream::new(input);
        assert_eq!(stream.prev_token(), Some("(int)"));
        assert_eq!(stream.prev_token(), Some("(* foo(int))"));
        assert_eq!(stream.prev_token(), Some("int"));
        assert_eq!(stream.prev_token(), None);

        let input = "* foo(int)";
        let mut stream = TokenStream::new(input);
        assert_eq!(stream.next_token(), Some("*"));
        assert_eq!(stream.next_token(), Some("foo"));
        assert_eq!(stream.next_token(), Some("(int)"));
        assert_eq!(stream.next_token(), None);
        let mut stream = TokenStream::new(input);
        assert_eq!(stream.prev_token(), Some("(int)"));
        assert_eq!(stream.prev_token(), Some("foo"));
        assert_eq!(stream.prev_token(), Some("*"));
        assert_eq!(stream.prev_token(), None);

        let input = "int[3] foo() const";
        let mut stream = TokenStream::new(input);
        assert_eq!(stream.next_token(), Some("int"));
        assert_eq!(stream.next_token(), Some("[3]"));
        assert_eq!(stream.next_token(), Some("foo"));
        assert_eq!(stream.next_token(), Some("()"));
        assert_eq!(stream.next_token(), Some("const"));
        assert_eq!(stream.next_token(), None);
        let mut stream = TokenStream::new(input);
        assert_eq!(stream.prev_token(), Some("const"));
        assert_eq!(stream.prev_token(), Some("()"));
        assert_eq!(stream.prev_token(), Some("foo"));
        assert_eq!(stream.prev_token(), Some("[3]"));
        assert_eq!(stream.prev_token(), Some("int"));
        assert_eq!(stream.prev_token(), None);

        let input = "int<void (*)(void), int<int>> ::std::foo<int[4], int(*)(), int<int>>(int<char*>, void (*f)(), int[4])";
        let mut stream = TokenStream::new(input);
        assert_eq!(stream.next_token(), Some("int"));
        assert_eq!(stream.next_token(), Some("<void (*)(void), int<int>>"));
        assert_eq!(
            stream.clone().next_path(),
            Some("::std::foo<int[4], int(*)(), int<int>>")
        );
        assert_eq!(stream.next_token(), Some(":"));
        assert_eq!(stream.next_token(), Some(":"));
        assert_eq!(stream.next_token(), Some("std"));
        assert_eq!(stream.next_token(), Some(":"));
        assert_eq!(stream.next_token(), Some(":"));
        assert_eq!(stream.next_token(), Some("foo"));
        assert_eq!(stream.next_token(), Some("<int[4], int(*)(), int<int>>"));
        assert_eq!(
            stream.next_token(),
            Some("(int<char*>, void (*f)(), int[4])")
        );
        assert_eq!(stream.next_token(), None);

        let mut stream = TokenStream::new(input);
        assert_eq!(
            stream.prev_token(),
            Some("(int<char*>, void (*f)(), int[4])")
        );
        assert_eq!(
            stream.clone().prev_path(),
            Some("::std::foo<int[4], int(*)(), int<int>>")
        );
        assert_eq!(stream.prev_token(), Some("<int[4], int(*)(), int<int>>"));
        assert_eq!(stream.prev_token(), Some("foo"));
        assert_eq!(stream.prev_token(), Some(":"));
        assert_eq!(stream.prev_token(), Some(":"));
        assert_eq!(stream.prev_token(), Some("std"));
        assert_eq!(stream.prev_token(), Some(":"));
        assert_eq!(stream.prev_token(), Some(":"));
        assert_eq!(stream.prev_token(), Some("<void (*)(void), int<int>>"));
        assert_eq!(stream.prev_token(), Some("int"));
        assert_eq!(stream.prev_token(), None);
    }
}
