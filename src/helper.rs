use rustyline_derive::{Helper, Validator, Completer};
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::Hinter;
use rustyline::history::SearchDirection;
use rustyline::{Context, ConditionalEventHandler, KeyEvent, EventContext, RepeatCount, Event, Cmd};
use std::borrow::Cow::{self, Borrowed, Owned};
use rustyline::hint::HistoryHinter;

#[derive(Helper, Validator, Completer)]
pub struct MyHelper {
    pub highlighter: MatchingBracketHighlighter,
    #[rustyline(Hinter)]
    pub hinter: HistoryHinter,
}
impl Highlighter for MyHelper {
    
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}
impl Hinter for MyHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<String> {
        if line.is_empty() || pos < line.len() {
            return None;
        }
        let start = if _ctx.history_index() == _ctx.history().len() {
            _ctx.history_index().saturating_sub(1)
        } else {
            _ctx.history_index()
        };
        if let Some(sr) = _ctx
            .history()
            .starts_with(line, start, SearchDirection::Reverse)
        {
            if sr.entry == line {
                return None;
            }
            return Some(sr.entry[pos..].to_owned());
        }
        None
    }
}

pub struct TabEventHandler;
impl ConditionalEventHandler for TabEventHandler {
    fn handle(&self, evt: &Event, n: RepeatCount, _: bool, ctx: &EventContext) -> Option<Cmd> {
        debug_assert_eq!(*evt, Event::from(KeyEvent::from('\t')));

        if ctx.line()[..ctx.pos()]
            .chars()
            .rev()
            .next()
            .filter(|c| c.is_whitespace())
            .is_some()
        {
            Some(Cmd::SelfInsert(n, '\t'))
        } else {
            None // default complete
        }
    }
}
impl MyHelper {
    pub fn new() -> MyHelper {
        MyHelper{
            highlighter : MatchingBracketHighlighter::new(),
            hinter : HistoryHinter {}
        }
    }
}