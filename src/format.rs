use crate::{
    code_blocks::{Block, Blocks, LangBlocks, Language},
    config::Config,
};

use memchr::memmem::Finder;
use std::{collections::HashMap, io::Write};

// format code blocks contained in `data` and write full
// output to the passed in writer.
pub fn format<W: Write>(config: &Config, data: &str, writer: W) {
    let (blocks, map) = get_code_blocks(data, config);

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

fn get_code_blocks<'a, 'b>(
    data: &'a str,
    config: &'b Config,
) -> (Blocks<'a>, HashMap<Language<'b>, LangBlocks<'b>>) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_code_blocks() {
        let config = Config::default();
        let data = r#"
            test markdown data

            ```rust
                fn testing(){}
            ```
        "#;
        let (mut blocks, map) = get_code_blocks(data, &config);
        assert_eq!(blocks.items.len(), 1);
        assert_eq!(map.len(), 1);

        let code = blocks.items[0].get_mut();
        assert_eq!(code.start, 53);
        assert_eq!(code.end, 96);
    }

    #[test]
    fn test_format() {
        let config = Config::default();
        let data = r#"
Cillum culpa aliquip non aute nostrud adipisicing.
Qui irure ullamco anim est irure qui mollit amet irure fugiat consectetur tempor.

```rust
    fn testing(){ println!("test");}
```

Minim velit mollit Lorem ad. Esse minim sint aute ex proident. Quis ut excepteur do esse sint proident velit culpa tempor occaecat.
Irure elit deserunt aute in. Elit dolore quis aliqua. Eu consectetur ipsum nostrud enim sunt excepteur voluptate qui aliqua nostrud aliqua irure amet esse.

```zig
fn test2() void { const num = 99; std.log.info("test: {}",.{num}); }
```

end
"#;

        let mut buff = Vec::new();
        format(&config, data, &mut buff);
        let output = String::from_utf8(buff).unwrap();
        assert_eq!(
            output,
            r#"
Cillum culpa aliquip non aute nostrud adipisicing.
Qui irure ullamco anim est irure qui mollit amet irure fugiat consectetur tempor.

```rust
fn testing() {
    println!("test");
}
```

Minim velit mollit Lorem ad. Esse minim sint aute ex proident. Quis ut excepteur do esse sint proident velit culpa tempor occaecat.
Irure elit deserunt aute in. Elit dolore quis aliqua. Eu consectetur ipsum nostrud enim sunt excepteur voluptate qui aliqua nostrud aliqua irure amet esse.

```zig
fn test2() void {
    const num = 99;
    std.log.info("test: {}", .{num});
}
```

end
"#
        );
    }
}
