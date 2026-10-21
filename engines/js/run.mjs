// JavaScript and wasm engines for the projections suite: every projection in
// suites/projections/reference.json through every engine, one JSON line per
// (projection, engine), the same format as engines/rust-projections.
//
//   node run.mjs ../../suites/projections/reference.json
//   node run.mjs -e math.gl -p ortho ../../suites/projections/reference.json
//
// Engines: proj4js (npm proj4), math.gl (@math.gl/projection, the deck.gl v10
// path) and proj4rs-wasm (the proj4rs main branch built by
// build-proj4rs-wasm.sh; skipped if proj4rs-pkg/ is missing). Nothing is
// judged here: report/projections.py judges every engine with the same code.

import fs from 'node:fs';
import {createRequire} from 'node:module';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);
const version = name => JSON.parse(fs.readFileSync(path.join(here, 'node_modules', name, 'package.json'))).version;

const engines = [];

{
  const proj4 = (await import('proj4')).default;
  engines.push({
    name: 'proj4js',
    version: version('proj4'),
    prepare(p) {
      const t = proj4(p.geographic, p.proj);
      return {fwd: (lon, lat) => t.forward([lon, lat]), inv: (x, y) => t.inverse([x, y])};
    }
  });
}

{
  const {projectionEngine} = await import('@math.gl/projection');
  engines.push({
    name: 'math.gl',
    version: version('@math.gl/projection'),
    prepare(p) {
      const t = projectionEngine.createProjection({from: p.geographic, to: p.proj});
      return {fwd: (lon, lat) => t.project([lon, lat]), inv: (x, y) => t.unproject([x, y])};
    }
  });
}

const pkg = path.join(here, 'proj4rs-pkg', 'proj4rs.js');
if (fs.existsSync(pkg)) {
  const P = require(pkg);
  const rev = fs.readFileSync(path.join(here, 'proj4rs-pkg', 'REV'), 'utf8').trim();
  engines.push({
    name: 'proj4rs-wasm',
    version: `git ${rev}`,
    prepare(p) {
      const geo = new P.Projection(p.geographic);
      const prj = new P.Projection(p.proj);
      const go = (a, b, x, y) => {
        const pt = new P.Point(x, y, 0);
        P.transform(a, b, pt);
        const out = [pt.x, pt.y];
        pt.free();
        return out;
      };
      return {fwd: (lon, lat) => go(geo, prj, lon, lat), inv: (x, y) => go(prj, geo, x, y)};
    }
  });
}

function args() {
  const a = {engines: [], only: [], file: 'suites/projections/reference.json'};
  const v = process.argv.slice(2);
  for (let i = 0; i < v.length; i++) {
    if (v[i] === '-e' || v[i] === '--engine') a.engines.push(v[++i]);
    else if (v[i] === '-p' || v[i] === '--projection') a.only.push(v[++i]);
    else if (v[i] === '--list') a.list = true;
    else a.file = v[i];
  }
  return a;
}

const finite = v => Array.isArray(v) && Number.isFinite(v[0]) && Number.isFinite(v[1]) ? [v[0], v[1]] : null;
const ascii = s => String(s).replace(/[^\x00-\x7f]/g, '?');

function attempt(f) {
  try {
    return finite(f());
  } catch {
    return null;
  }
}

const a = args();
if (a.list) {
  for (const e of engines) console.log(e.name.padEnd(16), e.version);
  process.exit(0);
}
const ref = JSON.parse(fs.readFileSync(a.file, 'utf8'));
if (ref.environment.schema_version !== 1) {
  console.error(`${a.file}: schema_version ${ref.environment.schema_version}, this runner reads 1`);
  process.exit(1);
}
for (const e of engines) {
  if (a.engines.length && !a.engines.includes(e.name)) continue;
  for (const p of ref.projections) {
    if (a.only.length && !a.only.includes(p.id)) continue;
    const line = {projection: p.id, engine: e.name, engine_version: e.version};
    let t;
    try {
      t = e.prepare(p);
    } catch (err) {
      console.log(JSON.stringify({...line, supported: false, error: ascii(err.message ?? err)}));
      continue;
    }
    const fwd = p.points.map(([lon, lat]) => attempt(() => t.fwd(lon, lat)));
    const inv = (p.fwd ?? []).map(xy => (xy ? attempt(() => t.inv(xy[0], xy[1])) : null));
    console.log(JSON.stringify({...line, supported: true, fwd, inv}));
  }
}
