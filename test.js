const binding = require('./get-binding');
const path = require('path');

function normalizeOptions(dir, opts = {}) {
  if (Array.isArray(opts.ignore)) {
    opts = Object.assign({}, opts, {
      ignore: opts.ignore.map((ignore) => path.resolve(dir, ignore)),
    });
  }

  return opts;
}

async function run() {
  let dir = path.resolve('.');
  let opts = normalizeOptions(dir, {});
  let callback = (...params) => {
    console.log(params);
  };
  await binding.subscribe(dir, callback, opts);

  return {
    unsubscribe() {
      return binding.unsubscribe(dir, callback, opts);
    },
  };
}

run().catch((err) => console.error(err));
