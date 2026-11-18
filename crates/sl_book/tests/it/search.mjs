// Runs the built book's own search worker (assets/javascripts/workers/search.*.js) on its search.json, as the
// browser does, and prints the first hit of each query as one JSON line: {query, location, title, text}.
// Usage: node search.mjs SITE_DIR QUERY...
import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";

const [site, ...queries] = process.argv.slice(2);
const workers = path.join(site, "assets/javascripts/workers");
const worker = fs.readdirSync(workers).find((f) => /^search\..*\.js$/.test(f));
if (!worker) throw new Error(`no search worker in ${workers}`);
const index = JSON.parse(fs.readFileSync(path.join(site, "search.json"), "utf8"));

// The worker talks through self.onmessage and self.postMessage; each request gets exactly one answer.
let answer;
const self = { postMessage: (m) => answer(m) };
vm.runInContext(
  fs.readFileSync(path.join(workers, worker), "utf8"),
  vm.createContext({ self, console, setTimeout, queueMicrotask, TextEncoder, TextDecoder, Intl }),
);
const ask = (data) => new Promise((resolve) => { answer = resolve; self.onmessage({ data }); });

// Message types of the worker: 0 sets up the index (answer 1), 2 is a query (answer 3), 4 an error.
const ready = await ask({ type: 0, data: index });
if (ready.type !== 1) throw new Error(`the search index did not load: ${JSON.stringify(ready)}`);
for (const query of queries) {
  // The query as the search box sends it, with no tag filter
  const r = await ask({
    type: 2,
    data: {
      input: query,
      filter: { input: { type: "operator", data: { operator: "and", operands: [] } } },
      aggregation: { input: [{ type: "term", data: { field: "tags" } }] },
    },
  });
  if (r.type !== 3) throw new Error(`query ${query} failed: ${JSON.stringify(r)}`);
  const first = r.data.items[0];
  const item = first === undefined ? {} : index.items[first.id];
  console.log(JSON.stringify({ query, location: item.location ?? null, title: item.title ?? null, text: item.text ?? null }));
}
