use colored::Colorize;
use rustyline::completion::{Completer, Pair, Candidate};
use rustyline_derive::{Helper, Validator, Hinter};
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::{Context, ConditionalEventHandler, KeyEvent, EventContext, RepeatCount, Event, Cmd};
use std::borrow::Cow::{self, Borrowed, Owned};
use rustyline::hint::HistoryHinter;

#[derive(Helper, Validator, Hinter)]
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

impl Completer for MyHelper {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize,ctx: &Context<'_> ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let candidates : Vec<&String> = ctx
            .history()
            .iter()
            .filter(|str| str.starts_with(line) && str.as_str() != line)
            .collect();

            Ok((
                pos,
                candidates
                    .iter()
                    .map(|&candidate| Pair {
                        // FIXME move this to highlight_candidate when that accepts a completion::Candidate
                        display: format!(
                            "{}{}",
                            &candidate[..pos].green(),
                            &candidate[pos..].bright_green().bold()
                        ),
                        replacement: if candidates.len() == 1 {
                            format!("{} ", &candidate[pos..])
                        } else {
                            candidate[pos..].to_owned()
                        },
                    })
                    .collect(),
            ))        
            
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