const fs = require("fs");
const b = require("binaryen");
const uleb = require("leb128/unsigned");

const blob = fs.readFileSync(process.argv[2], null);

const m = b.readBinary(blob);

const getBranches = (id, out) => {
	if(id==0)
		return;
	const expr = b.getExpressionInfo(id);
	if (expr.id==b.BreakId||expr.id==b.IfId) {
		out.push(expr);
	}
	switch(expr.id) {
		case b.BlockId: {
			for (let c of expr.children) {
				getBranches(c, out);
			}
			break;
		}
		case b.LoopId: {
			getBranches(expr.body, out);
			break;
		}
		case b.IfId: {
			getBranches(expr.ifTrue, out);
			getBranches(expr.ifFalse, out);
			break;
		}
	}
}

let info = [];
for (let i = 0; i < m.getNumFunctions(); i++) {
	let out = [];
	const f = m.getFunctionByIndex(i);
	const bodyId = b.getFunctionInfo(f).body;
	getBranches(bodyId, out);
	info.push(out);
}


const u32toULEB = (u) => {
	const buf = uleb.encode(u);
	return [...buf];
}

const append = (arr1, arr2) => {
	arr1.push(...arr2);
}
const data = [];

for (let [fidx, fentry] of info.entries()) {
	if(fentry.length==0)
		continue;
	append(data, u32toULEB(fidx));
	append(data, u32toULEB(fentry.length));
	for (let [idx, entry] of fentry.entries()) {
		append(data, u32toULEB(1));
	}
}

const section = new Uint8Array(data);

m.addCustomSection("branchHints", section);

let binary = m.emitBinary();

fs.writeFileSync(process.argv[3], binary);


