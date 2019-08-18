use ::std::{
    collections::{
        BTreeMap,
        BTreeSet,
    },
    ops::Not,
};

#[derive(
    Debug,
    Default,
    Clone, Copy,
    PartialEq, Eq,
    PartialOrd, Ord,
)]
pub
struct LineColumn {
    line: usize,
    column: usize,
}

#[derive(
    Debug,
    Default,
    Clone, Copy,
    PartialEq, Eq,
)]
pub
struct Span {
    start: LineColumn,
    end: LineColumn,
}

// #[derive(
//     Debug,
//     Clone, Copy,
//     PartialEq, Eq,
// )]
// struct Comment {
//     span: Span,
//     kind: CommentKind,
// }

pub
fn find_comments (input: &'_ str)
    -> BTreeMap<LineColumn, Span>
{

    let mut ret = BTreeMap::new();
    let mut yield_span = |span: Span| {
        ret.insert(span.start, span);
    };

    #[derive(Clone, Copy)]
    enum State {
        Code,
        LineComment { start: LineColumn, end: LineColumn },
        BlockComment { start: LineColumn },
        StringLiteral { pounds: usize },
    }

    let mut state = State::Code;

    input.lines().enumerate().for_each(|(i, line)| {
        if let State::LineComment { start, end } = state {
            yield_span(Span { start, end });
            state = State::Code;
        }
        let mut iterator = /* ::itertools::multipeek*/(
            line.char_indices()
                .map(|(j, c)| {
                    (LineColumn { line: i + 1, column: j + 1 }, c)
                })
                .peekable()
        );
        while let Some((pos, c)) = iterator.next() {
            match (&mut state, c) {
                | (&mut State::Code, '"') => {
                    state = State::StringLiteral { pounds: 0 };
                },

                | (&mut State::Code, 'r') => match iterator.peek() {
                    | Some((_, '"')) => {
                        let _ = iterator.next();
                        state = State::StringLiteral { pounds: 0 }
                    },
                    | Some((_, '#')) => {
                        let _ = iterator.next();
                        let mut pounds = 1;
                        loop {
                            match iterator.peek() {
                                | Some(&(_, '#')) => {
                                    let _ = iterator.next();
                                    pounds += 1;
                                    continue;
                                },
                                | Some(&(_, '"')) => {
                                    let _ = iterator.next();
                                    state = State::StringLiteral { pounds };
                                },
                                | _ => {},
                            }
                            break;
                        }
                    },
                    | _ => {},
                },

                | (&mut State::Code, '/') => match iterator.peek() {
                    // `//`
                    | Some(&(_, '/')) => {
                        let _ = iterator.next();
                        let is_doc_comment = 
                            if let Some(&(_, '/')) = iterator.peek() {
                                let _ = iterator.next();
                                if let Some(&(_, '/')) = iterator.peek() {
                                    let _ = iterator.next();
                                    false /* //// */
                                } else {
                                    true /* /// */
                                }
                            } else {
                                false /* // */
                            }
                        ;
                        if is_doc_comment.not() {
                            let (start, end) = (pos, pos);
                            state = State::LineComment { start, end };
                        }
                    },

                    // `/*`
                    | Some(&(_, '*')) => {
                        let _ = iterator.next();
                        let is_doc_comment =
                            if let Some(&(_, '*')) = iterator.peek() {
                                let _ = iterator.next();
                                match iterator.peek() {
                                    // `/***`
                                    | Some(&(_, '*')) => {
                                        let _ = iterator.next();
                                        false
                                    }
                                    // `/**/`
                                    | Some(&(_, '/')) => {
                                        let _ = iterator.next();
                                        let mut end = pos;
                                        end.column += 3;
                                        yield_span(Span { start: pos, end });
                                        continue;
                                    },
                                    // `/**`
                                    | _ => {
                                        true
                                    }
                                }
                            } else {
                                false // `/*`
                            }
                        ;
                        if is_doc_comment.not() {
                            state = State::BlockComment { start: pos };
                        }
                    },

                    // `/_`
                    | _ => {},
                },

                | (&mut State::BlockComment { start }, '*') => {
                    if let Some(&(_, '/')) = iterator.peek() {
                        // `*/`
                        let _ = iterator.next();
                        let mut end = pos;
                        end.column += 1;
                        yield_span(Span { start, end });
                        state = State::Code;
                    }
                },

                | (&mut State::LineComment { ref mut end, .. }, _) => {
                    *end = pos;
                },

                | (&mut State::StringLiteral { pounds }, '"') => {
                    if  (0 .. pounds)
                            .all(|_| if let Some((_, '#')) = iterator.peek() {
                                let _ = iterator.next();
                                true
                            } else {
                                false
                            })
                    {
                        state = State::Code;
                    }
                },

                | _ => {},
            }
        }
    });
    if let State::LineComment { start, end } = state {
        yield_span(Span { start, end });
    }
    ret
}
