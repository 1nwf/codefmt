use std::{
    cell::UnsafeCell,
    io::Write,
    process::{Command, Stdio},
};

use crate::config::LanguageConfig;

#[derive(Clone)]
pub struct Language<'a> {
    name: &'a str,
    cfg: &'a LanguageConfig,
}

impl PartialEq for Language<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Eq for Language<'_> {}
impl std::hash::Hash for Language<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl<'a> Language<'a> {
    pub fn new(name: &'a str, cfg: &'a LanguageConfig) -> Self {
        Self { name, cfg }
    }
}

// this is the separator that will be inserted in between
// code blocks that share the same language.
const SEP: &'static str = "__mdfmt__\n";

#[derive(Debug)]
pub struct Block<'a> {
    pub start: usize,
    pub end: usize,
    pub data: &'a str,
}

pub type BlockIdx = u16;

pub struct LangBlocks<'a> {
    pub lang: Language<'a>,
    pub blocks: Vec<BlockIdx>,
    pub joined_data: String,
}

impl<'a> LangBlocks<'a> {
    pub fn new(lang: Language<'a>) -> Self {
        Self {
            lang,
            blocks: Vec::new(),
            joined_data: String::new(),
        }
    }

    pub fn add(&mut self, data: &str, idx: BlockIdx) {
        self.blocks.push(idx);

        self.joined_data.push_str(&self.lang.cfg.comment_token);
        self.joined_data.push_str(SEP);
        self.joined_data.push_str(data);
    }

    // spawn language format command and update block data
    pub fn format(&self, blocks: &Blocks) {
        let args = &self.lang.cfg.formatter;
        let mut cmd = Command::new(&args[0])
            .args(&args[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        cmd.stdin
            .as_mut()
            .unwrap()
            .write_all(self.joined_data.as_bytes())
            .unwrap();

        let output = cmd.wait_with_output().unwrap();

        let mut buffer = [0u8; 24];
        let sep = {
            let comment_token = &self.lang.cfg.comment_token;
            buffer[0..comment_token.len()].copy_from_slice(comment_token.as_bytes());
            buffer[comment_token.len()..comment_token.len() + SEP.len()]
                .copy_from_slice(SEP.as_bytes());
            unsafe { str::from_utf8_unchecked(&buffer[0..SEP.len() + comment_token.len()]) }
        };

        let stdout = String::from_utf8(output.stdout).unwrap().leak();
        let iter = stdout.split(sep);

        let mut idx = 0;
        for data in iter.filter(|x| x.len() > 0) {
            // SAFETY: Indices in each `LangBlock` object are guaranteed to be unique.
            // As a result, this is safe because no two `LangBlock` instances will
            // contain the same index into `Blocks` items.
            unsafe { blocks.get_mut(self.blocks[idx]).data = data }
            idx += 1;
        }
    }
}

pub struct Blocks<'a> {
    pub items: Vec<UnsafeCell<Block<'a>>>,
}

impl<'a> Blocks<'a> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn push(&mut self, block: Block<'a>) -> BlockIdx {
        let idx = self.items.len();
        self.items.push(UnsafeCell::new(block));
        idx as BlockIdx
    }

    pub unsafe fn get_mut(&self, idx: BlockIdx) -> &mut Block<'a> {
        unsafe { &mut *self.items[idx as usize].get() }
    }
}

unsafe impl Sync for Blocks<'_> {}
