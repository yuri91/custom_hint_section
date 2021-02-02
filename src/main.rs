use anyhow::Result;
use std::env;
use std::io::Write;
use wasmparser::{Parser, Payload, Operator};

#[derive(Debug, PartialEq, Eq)]
enum HintDirection {
    True,
    False,
}
#[derive(Debug)]
struct BranchInfo {
    dir: HintDirection,
    offset: u32,
}
#[derive(Debug)]
struct FuncBranchInfo {
    func: u32,
    branches: Vec<BranchInfo>,
}
#[derive(Debug, Default)]
struct BranchHintsSection {
    funcs: Vec<FuncBranchInfo>,
}

impl BranchHintsSection {
    fn write(&self) -> Vec<u8> {
        let mut data = Vec::new();
        let name = b"branchHints";
        leb128::write::unsigned(&mut data, name.len() as u64).unwrap();
        data.extend(name);
        for f in &self.funcs {
            leb128::write::unsigned(&mut data, f.func as u64).unwrap();
            leb128::write::unsigned(&mut data, f.branches.len() as u64).unwrap();
            for i in &f.branches {
                leb128::write::unsigned(&mut data, i.offset as u64).unwrap();
                let direction = match i.dir {
                    HintDirection::False => {
                        0
                    },
                    HintDirection::True => {
                        1
                    }
                };
                leb128::write::unsigned(&mut data, direction).unwrap();
            }
        }
        let mut payload = Vec::new();
        payload.push(0);
        leb128::write::unsigned(&mut payload, data.len() as u64).unwrap();
        payload.append(&mut data);
        payload
    }
}

fn main() -> Result<()> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 3 {
        println!("Usage: {} in.wasm out.wasm", args[0]);
        return Ok(());
    }

    let buf: Vec<u8> = std::fs::read(&args[1])?;
    let mut cur_func_idx = 0;
    let mut section = BranchHintsSection{funcs: Vec::new()};
    let mut code_start = 0;
    for payload in Parser::new(0).parse_all(&buf) {
        match payload? {
            Payload::ImportSection(reader) => {
                cur_func_idx = reader.get_count();
            }
            Payload::CodeSectionStart{range,size, ..} => {
                let mut size_buf = Vec::new();
                leb128::write::unsigned(&mut size_buf, size as u64)?;
                code_start = range.start - 1 - size_buf.len();
                println!("code start {:x} {:?}", code_start, &buf[code_start..range.end]);
            }
            Payload::CodeSectionEntry(body) => {
                let op_reader = body.get_operators_reader()?;
                let offset_start = op_reader.original_position() as u32;
                println!("offset_start {:x}", offset_start);
                let mut info = FuncBranchInfo{func: cur_func_idx, branches: Vec::new()};
                for r in op_reader.into_iter_with_offsets() {
                    let (op, off) = r?;
                    println!("{:?} {:x}", op, off);
                    let off = off as u32;
                    let dir = match op {
                        Operator::BrIf{..} => {
                            HintDirection::True
                        }
                        Operator::If{..} => {
                            HintDirection::False
                        }
                        _ => {
                            continue;
                        }
                    };
                    info.branches.push(BranchInfo{dir, offset:off-offset_start});
                }
                if !info.branches.is_empty() {
                    section.funcs.push(info);
                }
            }
            _ => {
            }
        }
    }
    println!("{:?}", section);
    let section_bytes = section.write();
    let mut out: Vec<u8> = Vec::with_capacity(buf.len()+section_bytes.len());
    out.extend(&buf[..code_start]);
    out.extend(&section_bytes);
    out.extend(&buf[code_start..]);

    let mut file = std::fs::File::create(&args[2])?;
    file.write_all(&out)?;

    Ok(())
}
