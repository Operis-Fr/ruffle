use swf::avm2::read::Reader;
use swf::avm2::types::*;
use swf::extensions::ReadSwfExt;

fn s(abc: &AbcFile, i: u32) -> String {
    if i == 0 { return "*".into(); }
    String::from_utf8_lossy(&abc.constant_pool.strings[i as usize - 1]).into_owned()
}
fn mn(abc: &AbcFile, i: u32) -> String {
    if i == 0 { return "*".into(); }
    match &abc.constant_pool.multinames[i as usize - 1] {
        Multiname::QName { name, .. } | Multiname::QNameA { name, .. } => s(abc, name.0),
        Multiname::Multiname { name, .. } | Multiname::MultinameA { name, .. } => s(abc, name.0),
        Multiname::RTQName { name } | Multiname::RTQNameA { name } => s(abc, name.0),
        other => format!("{:?}", other),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&args[1]).unwrap();
    let swf_buf = swf::decompress_swf(&data[..]).unwrap();
    let swf = swf::parse_swf(&swf_buf).unwrap();
    for tag in &swf.tags {
        let abc_data = match tag { swf::Tag::DoAbc(d) => *d, swf::Tag::DoAbc2(d) => d.data, _ => continue };
        let abc = Reader::new(abc_data).read().unwrap();
        for inst in &abc.instances {
            if mn(&abc, inst.name.0) != args[2] { continue; }
            for t in &inst.traits {
                let name = mn(&abc, t.name.0);
                if !args[3..].contains(&name) { continue; }
                if let TraitKind::Method { method, .. } = t.kind {
                    let body = abc.method_bodies.iter().find(|b| b.method.0 == method.0).unwrap();
                    println!("=== {name}");
                    let mut r = Reader::new(&body.code);
                    let total = body.code.len();
                    loop {
                        let pos = total - r.as_slice().len();
                        let Ok(op) = r.read_op() else { break };
                        let txt = format!("{:?}", op);
                        // resolve multiname indices
                        let txt = match &op {
                            swf::avm2::types::Op::GetProperty { index } | swf::avm2::types::Op::SetProperty { index }
                            | swf::avm2::types::Op::CallProperty { index, .. } | swf::avm2::types::Op::CallPropVoid { index, .. }
                            | swf::avm2::types::Op::GetLex { index } | swf::avm2::types::Op::FindPropStrict { index }
                            | swf::avm2::types::Op::InitProperty { index } | swf::avm2::types::Op::Coerce { index }
                            | swf::avm2::types::Op::FindProperty { index } | swf::avm2::types::Op::ConstructProp { index, .. }
                            => format!("{txt} ; {}", mn(&abc, index.0)),
                            swf::avm2::types::Op::PushString { value } => format!("PushString {:?}", s(&abc, value.0)),
                            _ => txt,
                        };
                        println!("{pos:5} {txt}");
                        if r.as_slice().is_empty() { break; }
                    }
                }
            }
        }
    }
}
