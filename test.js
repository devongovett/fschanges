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
  console.log(binding.ParcelWatcher);
  const watcherInstance = new binding.ParcelWatcher();
  console.log(watcherInstance);

  let dir = path.resolve('.');
  let opts = normalizeOptions(dir, {});
  let callback = (...params) => {
    console.log(params);
  };

  setInterval(() => {
    watcherInstance.process_events();
  }, 100);

  await watcherInstance.subscribe(dir, callback, opts);

  //   return {
  //     unsubscribe() {
  //       return watcherInstance.unsubscribe(dir, callback, opts);
  //     },
  //   };
}

run().catch((err) => console.error(err));
