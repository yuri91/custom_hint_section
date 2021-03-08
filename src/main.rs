use anyhow::Result;
use std::env;
use std::io::Write;
use std::collections::HashMap;
use wasmparser::{Parser, Payload, Operator};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
        leb128::write::unsigned(&mut data, self.funcs.len() as u64).unwrap();
        for f in &self.funcs {
            leb128::write::unsigned(&mut data, f.func as u64).unwrap();
            leb128::write::unsigned(&mut data, 0u64).unwrap();
            leb128::write::unsigned(&mut data, f.branches.len() as u64).unwrap();
            for i in &f.branches {
                let direction = match i.dir {
                    HintDirection::False => {
                        0
                    },
                    HintDirection::True => {
                        1
                    }
                };
                leb128::write::unsigned(&mut data, direction).unwrap();
                leb128::write::unsigned(&mut data, i.offset as u64).unwrap();
            }
        }
        let mut payload = Vec::new();
        payload.push(0);
        leb128::write::unsigned(&mut payload, data.len() as u64).unwrap();
        payload.append(&mut data);
        payload
    }
}

fn parse_file(path: &str) -> Result<HashMap<u32, HashMap<u32, HintDirection>>> {
    let content = std::fs::read_to_string(path)?;
    let mut ret: HashMap<u32, HashMap<_,_>> = HashMap::new();
    let mut cur = 0u32;
    for l in content.lines() {
        if l.starts_with('\t') {
            let space = l.find(' ').unwrap()-1;
            let (branch_n, branch_v) = l.trim_start_matches('\t').split_at(space);
            let hint = match branch_v.trim().parse::<u32>()? {
                0 => HintDirection::False,
                1 => HintDirection::True,
                _ => unreachable!()
            };
            ret.get_mut(&cur).unwrap().insert(branch_n.parse()?, hint);
        } else {
            cur = l.parse()?;
            ret.insert(cur, HashMap::new());
        }
    }
    Ok(ret)
}
fn main() -> Result<()> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 4 {
        println!("Usage: {} in.wasm out.wasm hints.txt", args[0]);
        return Ok(());
    }

    let buf: Vec<u8> = std::fs::read(&args[1])?;
    let hints = parse_file(&args[3])?;
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
            }
            Payload::CodeSectionEntry(body) => {
                let op_reader = body.get_operators_reader()?;
                let offset_start = op_reader.original_position() as u32;
                let mut info = FuncBranchInfo{func: cur_func_idx, branches: Vec::new()};
                for r in op_reader.into_iter_with_offsets() {
                    let (op, off) = r?;
                    let offset = off as u32 - offset_start;
                    let dir = match op {
                        Operator::BrIf{..}|Operator::If{..} => {
                            println!("f {} br {}", cur_func_idx, offset);
                            hints.get(&cur_func_idx).and_then(|m| m.get(&offset))
                        }
                        _ => {
                            continue;
                        }
                    };
                    let dir = match dir {
                        Some(d) => d,
                        None => continue
                    };
                    info.branches.push(BranchInfo{dir:*dir, offset});
                }
                if !info.branches.is_empty() {
                    section.funcs.push(info);
                }
                cur_func_idx+=1;
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
