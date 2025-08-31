use crate::{
    code_blocks::{Block, Blocks, LangBlocks, Language},
    config::Config,
};

use memchr::memmem::Finder;
use std::{collections::HashMap, io::Write};

// format code blocks contained in `data` and write full
// output to the passed in writer.
pub fn format<W: Write>(config: &Config, data: String, writer: W) {
    let (blocks, map) = get_code_blocks(config, &data);

    // spawn thread to run format command for each
    // language in the markdown data
    std::thread::scope(|s| {
        for (_, b) in map {
            s.spawn(|| {
                let block = b;
                block.format(&blocks);
            });
        }
    });

    // write output to writer
    let mut writer = std::io::BufWriter::new(writer);
    let mut start = 0;
    for block in blocks.items {
        let block = unsafe { &*block.get() };
        writer
            .write_all(&data[start..block.start].as_bytes())
            .unwrap();
        writer.write_all(&block.data.as_bytes()).unwrap();
        start = block.end;
    }
    writer.write_all(&data[start..].as_bytes()).unwrap();
}

fn get_code_blocks<'a>(
    config: &Config,
    data: &'a str,
) -> (Blocks<'a>, HashMap<Language, LangBlocks>) {
    let mut blocks = Blocks::new();
    let mut map = HashMap::<Language, LangBlocks>::new();

    const DELIM: &'static str = "```";
    let finder = Finder::new(DELIM);
    let mut iter = finder.find_iter(data.as_bytes());

    loop {
        let Some(block_start) = iter.next() else {
            break;
        };
        let Some(block_end) = iter.next() else {
            break;
        };

        let block_data = &data[block_start..block_end];
        let Some(nl) = block_data.find('\n') else {
            continue;
        };

        let code = &block_data[nl + 1..];
        let idx = blocks.push(Block {
            start: block_start + nl + 1,
            end: block_end,
            data: code,
        });

        let Some(lang_cfg) = config.get_lang(&block_data[DELIM.len()..nl]) else {
            continue;
        };

        let language_name = String::from(&block_data[DELIM.len()..nl]);
        let lang = Language::new(language_name, lang_cfg);

        if let Some(lb) = map.get_mut(&lang) {
            lb.add(code, idx);
        } else {
            let mut lb = LangBlocks::new(lang.clone());
            lb.add(code, idx);
            map.insert(lang, lb);
        }
    }

    return (blocks, map);
}
