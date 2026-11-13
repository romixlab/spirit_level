//! Synthetic ELFs for the fixtures of the lints that read the linked ELF: a fixture's `elf.toml` describes
//! sections, segments, symbols and Thumb functions (frame and calls), and [`write`] turns it into a little 32-bit
//! ELF, so no binary is committed.
//!
//! ```toml
//! machine = "arm"                  # or "riscv"
//! [[section]]                      # flags: a alloc, w write, x exec; nobits for .bss; load: the LMA
//! name = ".text"
//! addr = 0x0800_0100
//! flags = "ax"
//! [[function]]                     # Thumb code: push, sub sp, one bl per call, pop; in .text by default
//! name = "Reset"
//! addr = 0x0800_0100
//! frame = 16
//! calls = ["main"]
//! [[symbol]]                       # kind: object, func, notype
//! name = "_stack_start"
//! addr = 0x2000_1000
//! [[segment]]                      # optional: LOAD segments, else one per alloc section
//! vaddr = 0x0800_0000
//! memsz = 0x100
//! flags = "rwx"
//! ```

use sl_model::toml_edit::{DocumentMut, Item, Table};
use std::collections::BTreeMap;

/// An `elf.toml`: the ELF to write
#[derive(Debug, Default)]
pub struct Spec {
    machine: Option<String>,
    section: Vec<SectionSpec>,
    function: Vec<FunctionSpec>,
    symbol: Vec<SymbolSpec>,
    segment: Vec<SegmentSpec>,
}

#[derive(Debug)]
struct SectionSpec {
    name: String,
    addr: u64,
    size: Option<u64>,
    flags: String,
    nobits: bool,
    load: Option<u64>,
}

#[derive(Debug)]
struct FunctionSpec {
    name: String,
    addr: u64,
    frame: u32,
    calls: Vec<String>,
    section: Option<String>,
}

#[derive(Debug)]
struct SymbolSpec {
    name: String,
    addr: u64,
    size: u64,
    kind: Option<String>,
    section: Option<String>,
}

#[derive(Debug)]
struct SegmentSpec {
    vaddr: u64,
    paddr: Option<u64>,
    memsz: u64,
    filesz: Option<u64>,
    flags: String,
}

/// The keys of one table, read with the error naming the table and key
struct Keys<'a> {
    what: String,
    t: &'a Table,
}

impl Keys<'_> {
    fn check(&self, known: &[&str]) -> Result<(), String> {
        match self.t.iter().find(|(k, _)| !known.contains(k)) {
            Some((k, _)) => Err(format!("{}: unknown key {k}", self.what)),
            None => Ok(()),
        }
    }
    fn str(&self, k: &str) -> Result<Option<String>, String> {
        match self.t.get(k) {
            None => Ok(None),
            Some(v) => v
                .as_str()
                .map(|s| Some(s.to_string()))
                .ok_or_else(|| format!("{}: {k} is not a string", self.what)),
        }
    }
    fn req_str(&self, k: &str) -> Result<String, String> {
        self.str(k)?
            .ok_or_else(|| format!("{}: {k} is missing", self.what))
    }
    fn int(&self, k: &str) -> Result<Option<u64>, String> {
        match self.t.get(k) {
            None => Ok(None),
            Some(v) => v
                .as_integer()
                .and_then(|i| u64::try_from(i).ok())
                .map(Some)
                .ok_or_else(|| format!("{}: {k} is not a whole number", self.what)),
        }
    }
    fn req_int(&self, k: &str) -> Result<u64, String> {
        self.int(k)?
            .ok_or_else(|| format!("{}: {k} is missing", self.what))
    }
    fn bool(&self, k: &str) -> Result<bool, String> {
        match self.t.get(k) {
            None => Ok(false),
            Some(v) => v
                .as_bool()
                .ok_or_else(|| format!("{}: {k} is not a boolean", self.what)),
        }
    }
    fn strs(&self, k: &str) -> Result<Vec<String>, String> {
        match self.t.get(k) {
            None => Ok(Vec::new()),
            Some(v) => v
                .as_array()
                .and_then(|a| a.iter().map(|s| s.as_str().map(str::to_string)).collect())
                .ok_or_else(|| format!("{}: {k} is not a list of strings", self.what)),
        }
    }
}

fn tables<'a>(doc: &'a DocumentMut, key: &str) -> Result<Vec<Keys<'a>>, String> {
    match doc.get(key) {
        None => Ok(Vec::new()),
        Some(Item::ArrayOfTables(a)) => Ok(a
            .iter()
            .enumerate()
            .map(|(i, t)| Keys {
                what: format!("[[{key}]] {}", i + 1),
                t,
            })
            .collect()),
        Some(_) => Err(format!("{key} is not an array of tables ([[{key}]])")),
    }
}

impl Spec {
    /// Reads an `elf.toml`
    pub fn parse(text: &str) -> Result<Spec, String> {
        let doc: DocumentMut = text.parse().map_err(|e| format!("elf.toml: {e}"))?;
        let top = Keys {
            what: "elf.toml".into(),
            t: doc.as_table(),
        };
        top.check(&["machine", "section", "function", "symbol", "segment"])?;
        let mut spec = Spec {
            machine: top.str("machine")?,
            ..Spec::default()
        };
        for k in tables(&doc, "section")? {
            k.check(&["name", "addr", "size", "flags", "nobits", "load"])?;
            spec.section.push(SectionSpec {
                name: k.req_str("name")?,
                addr: k.req_int("addr")?,
                size: k.int("size")?,
                flags: k.str("flags")?.unwrap_or_default(),
                nobits: k.bool("nobits")?,
                load: k.int("load")?,
            });
        }
        for k in tables(&doc, "function")? {
            k.check(&["name", "addr", "frame", "calls", "section"])?;
            spec.function.push(FunctionSpec {
                name: k.req_str("name")?,
                addr: k.req_int("addr")?,
                frame: k.int("frame")?.unwrap_or(0) as u32,
                calls: k.strs("calls")?,
                section: k.str("section")?,
            });
        }
        for k in tables(&doc, "symbol")? {
            k.check(&["name", "addr", "size", "kind", "section"])?;
            spec.symbol.push(SymbolSpec {
                name: k.req_str("name")?,
                addr: k.req_int("addr")?,
                size: k.int("size")?.unwrap_or(0),
                kind: k.str("kind")?,
                section: k.str("section")?,
            });
        }
        for k in tables(&doc, "segment")? {
            k.check(&["vaddr", "paddr", "memsz", "filesz", "flags"])?;
            spec.segment.push(SegmentSpec {
                vaddr: k.req_int("vaddr")?,
                paddr: k.int("paddr")?,
                memsz: k.req_int("memsz")?,
                filesz: k.int("filesz")?,
                flags: k.req_str("flags")?,
            });
        }
        Ok(spec)
    }
}

struct Sym {
    name: String,
    value: u64,
    size: u64,
    kind: u8,
    section: Option<String>,
}

fn put16(b: &mut Vec<u8>, v: u16) {
    b.extend_from_slice(&v.to_le_bytes());
}

fn put32(b: &mut Vec<u8>, v: u64) {
    b.extend_from_slice(&(v as u32).to_le_bytes());
}

/// The ELF the spec describes; fails on a call to a function the spec does not have
pub fn write(spec: &Spec) -> Result<Vec<u8>, String> {
    let arm = spec.machine.as_deref().unwrap_or("arm") == "arm";
    let fn_addr: BTreeMap<&str, u64> = spec
        .function
        .iter()
        .map(|f| (f.name.as_str(), f.addr))
        .collect();
    // code per function, then section contents
    let mut code: Vec<(String, u64, Vec<u8>)> = Vec::new();
    for f in &spec.function {
        let calls: Vec<u64> = f
            .calls
            .iter()
            .map(|c| {
                fn_addr
                    .get(c.as_str())
                    .copied()
                    .ok_or_else(|| format!("function {}: no function {c} to call", f.name))
            })
            .collect::<Result<_, _>>()?;
        let sec = f.section.clone().unwrap_or_else(|| ".text".into());
        code.push((sec, f.addr, crate::thumb::encode::function(f.addr, f.frame, &calls)));
    }
    struct Sec {
        name: String,
        addr: u64,
        size: u64,
        flags: u64,
        nobits: bool,
        load: u64,
        data: Vec<u8>,
        offset: u64,
    }
    let mut secs: Vec<Sec> = spec
        .section
        .iter()
        .map(|s| {
            let end = code
                .iter()
                .filter(|(n, ..)| *n == s.name)
                .map(|(_, a, c)| a + c.len() as u64)
                .max()
                .unwrap_or(s.addr);
            let size = s.size.unwrap_or(end - s.addr).max(end - s.addr);
            let mut data = if s.nobits {
                Vec::new()
            } else {
                vec![0; size as usize]
            };
            for (_, a, c) in code.iter().filter(|(n, ..)| *n == s.name) {
                let off = (a - s.addr) as usize;
                data[off..off + c.len()].copy_from_slice(c);
            }
            let mut flags = 0;
            for ch in s.flags.chars() {
                flags |= match ch {
                    'w' => 1,
                    'a' => 2,
                    'x' => 4,
                    _ => 0,
                };
            }
            Sec {
                name: s.name.clone(),
                addr: s.addr,
                size,
                flags,
                nobits: s.nobits,
                load: s.load.unwrap_or(s.addr),
                data,
                offset: 0,
            }
        })
        .collect();

    let mut syms: Vec<Sym> = Vec::new();
    for (f, (sec, _, c)) in spec.function.iter().zip(&code) {
        syms.push(Sym {
            name: f.name.clone(),
            value: if arm { f.addr | 1 } else { f.addr },
            size: c.len() as u64,
            kind: 2,
            section: Some(sec.clone()),
        });
    }
    for s in &spec.symbol {
        let section = s.section.clone().or_else(|| {
            secs.iter()
                .find(|x| s.addr >= x.addr && s.addr < x.addr + x.size.max(1))
                .map(|x| x.name.clone())
        });
        let kind = match s.kind.as_deref() {
            Some("object") => 1,
            Some("func") => 2,
            _ => 0,
        };
        syms.push(Sym {
            name: s.name.clone(),
            value: if kind == 2 && arm { s.addr | 1 } else { s.addr },
            size: s.size,
            kind,
            section,
        });
    }

    // segments: as given, or one per alloc section with a size
    let flag_bits = |f: &str| {
        f.chars().fold(0u64, |acc, c| {
            acc | match c {
                'x' => 1,
                'w' => 2,
                'r' => 4,
                _ => 0,
            }
        })
    };
    let mut phdrs: Vec<[u64; 6]> = Vec::new(); // vaddr, paddr, filesz, memsz, flags, section index (or MAX)
    if spec.segment.is_empty() {
        for (i, s) in secs.iter().enumerate() {
            if s.flags & 2 == 0 || s.size == 0 {
                continue;
            }
            let flags =
                4 | if s.flags & 1 != 0 { 2 } else { 0 } | if s.flags & 4 != 0 { 1 } else { 0 };
            let filesz = if s.nobits { 0 } else { s.size };
            phdrs.push([s.addr, s.load, filesz, s.size, flags, i as u64]);
        }
    } else {
        for g in &spec.segment {
            phdrs.push([
                g.vaddr,
                g.paddr.unwrap_or(g.vaddr),
                g.filesz.unwrap_or(g.memsz),
                g.memsz,
                flag_bits(&g.flags),
                u64::MAX,
            ]);
        }
    }

    // string tables
    let mut shstr = vec![0u8];
    let name_off = |names: &mut Vec<u8>, n: &str| {
        let o = names.len() as u64;
        names.extend_from_slice(n.as_bytes());
        names.push(0);
        o
    };
    let sec_names: Vec<u64> = secs.iter().map(|s| name_off(&mut shstr, &s.name)).collect();
    let symtab_name = name_off(&mut shstr, ".symtab");
    let strtab_name = name_off(&mut shstr, ".strtab");
    let shstrtab_name = name_off(&mut shstr, ".shstrtab");
    let mut strtab = vec![0u8];
    let sym_names: Vec<u64> = syms
        .iter()
        .map(|s| name_off(&mut strtab, &s.name))
        .collect();

    // layout
    let ehsize = 52u64;
    let mut off = ehsize + 32 * phdrs.len() as u64;
    for s in &mut secs {
        off = (off + 3) & !3;
        s.offset = off;
        off += s.data.len() as u64;
    }
    let mut symtab = Vec::new();
    symtab.extend_from_slice(&[0u8; 16]);
    for (s, &n) in syms.iter().zip(&sym_names) {
        put32(&mut symtab, n);
        put32(&mut symtab, s.value);
        put32(&mut symtab, s.size);
        symtab.push((1 << 4) | s.kind);
        symtab.push(0);
        let shndx = s
            .section
            .as_ref()
            .and_then(|n| secs.iter().position(|x| x.name == *n))
            .map_or(0xFFF1, |i| i as u16 + 1);
        put16(&mut symtab, shndx);
    }
    off = (off + 3) & !3;
    let symtab_off = off;
    off += symtab.len() as u64;
    let strtab_off = off;
    off += strtab.len() as u64;
    let shstr_off = off;
    off += shstr.len() as u64;
    off = (off + 3) & !3;
    let shoff = off;
    let shnum = secs.len() as u64 + 4;

    let mut b = Vec::new();
    b.extend_from_slice(&[0x7F, b'E', b'L', b'F', 1, 1, 1, 0]);
    b.extend_from_slice(&[0; 8]);
    put16(&mut b, 2);
    put16(&mut b, if arm { 40 } else { 243 });
    put32(&mut b, 1);
    put32(&mut b, spec.function.first().map_or(0, |f| f.addr | 1));
    put32(&mut b, if phdrs.is_empty() { 0 } else { ehsize });
    put32(&mut b, shoff);
    put32(&mut b, if arm { 0x0500_0200 } else { 0 });
    put16(&mut b, 52);
    put16(&mut b, 32);
    put16(&mut b, phdrs.len() as u16);
    put16(&mut b, 40);
    put16(&mut b, shnum as u16);
    put16(&mut b, shnum as u16 - 1);
    for p in &phdrs {
        let offset = if p[5] == u64::MAX {
            0
        } else {
            secs[p[5] as usize].offset
        };
        put32(&mut b, 1);
        put32(&mut b, offset);
        put32(&mut b, p[0]);
        put32(&mut b, p[1]);
        put32(&mut b, p[2]);
        put32(&mut b, p[3]);
        put32(&mut b, p[4]);
        put32(&mut b, 4);
    }
    for s in &secs {
        b.resize(s.offset as usize, 0);
        b.extend_from_slice(&s.data);
    }
    b.resize(symtab_off as usize, 0);
    b.extend_from_slice(&symtab);
    b.extend_from_slice(&strtab);
    b.extend_from_slice(&shstr);
    b.resize(shoff as usize, 0);
    let shdr = |b: &mut Vec<u8>, v: [u64; 10]| {
        for x in v {
            put32(b, x);
        }
    };
    shdr(&mut b, [0; 10]);
    for (s, &n) in secs.iter().zip(&sec_names) {
        let ty = if s.nobits { 8 } else { 1 };
        shdr(
            &mut b,
            [n, ty, s.flags, s.addr, s.offset, s.size, 0, 0, 4, 0],
        );
    }
    let strtab_idx = secs.len() as u64 + 2;
    shdr(
        &mut b,
        [
            symtab_name,
            2,
            0,
            0,
            symtab_off,
            symtab.len() as u64,
            strtab_idx,
            1,
            4,
            16,
        ],
    );
    shdr(
        &mut b,
        [
            strtab_name,
            3,
            0,
            0,
            strtab_off,
            strtab.len() as u64,
            0,
            0,
            1,
            0,
        ],
    );
    shdr(
        &mut b,
        [
            shstrtab_name,
            3,
            0,
            0,
            shstr_off,
            shstr.len() as u64,
            0,
            0,
            1,
            0,
        ],
    );
    Ok(b)
}
