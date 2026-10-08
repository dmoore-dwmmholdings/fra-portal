//! Map raw NTSB texts to dictionary nodes, for dictionary QA (spec §10).
//!
//! Input (stdin or file): TSV lines `src<TAB>count<TAB>text`, src = occ08 | find08 | occ_pre08 | find_pre08.
//! Output (stdout): TSV `src<TAB>count<TAB>node<TAB>text`; node is `(unmapped)` when nothing matches.
//! Texts are normalised the same way the adapter normalises them (`normalize_occ08` / `normalize_find08`).
//!
//! cargo run --release --bin dict-map -- texts.tsv [config/causal_nodes.toml]
use anyhow::{bail, Context, Result};
use causal_chain::{normalize_find08, normalize_occ08, Dict, Source};
use std::io::{BufRead, BufReader, Read};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let dict_path = args.get(1).map(String::as_str).unwrap_or("config/causal_nodes.toml");
    let dict = Dict::from_toml(&std::fs::read_to_string(dict_path).with_context(|| format!("read {dict_path}"))?)?;
    let input: Box<dyn Read> = match args.first().map(String::as_str) {
        None | Some("-") => Box::new(std::io::stdin()),
        Some(p) => Box::new(std::fs::File::open(p).with_context(|| format!("open {p}"))?),
    };
    for line in BufReader::new(input).lines() {
        let line = line?;
        let mut parts = line.splitn(3, '\t');
        let (Some(src), Some(count), Some(text)) = (parts.next(), parts.next(), parts.next()) else {
            bail!("bad line: {line}");
        };
        let (src_enum, text) = match src {
            "occ08" => (Source::Occ08, normalize_occ08(text)),
            "find08" => (Source::Find08, normalize_find08(text)),
            "occ_pre08" => (Source::OccPre08, text),
            "find_pre08" => (Source::FindPre08, text),
            _ => bail!("unknown src {src}"),
        };
        let node = dict.map(src_enum, text).map(|i| dict.nodes[i].id.as_str()).unwrap_or("(unmapped)");
        println!("{src}\t{count}\t{node}\t{text}");
    }
    Ok(())
}
