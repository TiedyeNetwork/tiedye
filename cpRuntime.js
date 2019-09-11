const fs = require('fs');

const tmpRaw = fs.readFileSync('./tmp_dev.json', { encoding: 'utf-8' });
const sunburstRaw = fs.readFileSync('./sunburst.json');
const tmp = JSON.parse(tmpRaw);
const sunburst = JSON.parse(sunburstRaw);
sunburst.genesis.runtime.system.code = tmp.genesis.runtime.system.code;
fs.writeFileSync('./sunburst.json', JSON.stringify(sunburst, null, 2));
