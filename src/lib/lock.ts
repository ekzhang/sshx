// Simple async lock for use in streaming encryption.
// See <https://stackoverflow.com/a/74538176>.
export function createLock() {
  const queue: (() => Promise<void>)[] = [];
  let active = false;
  return (fn: () => Promise<void>) => {
    let deferredResolve: any;
    let deferredReject: any;
    const deferred = new Promise((resolve, reject) => {
      deferredResolve = resolve;
      deferredReject = reject;
    });
    const exec = async () => {
      await fn().then(deferredResolve, deferredReject);
      if (queue.length > 0) {
        queue.shift()!();
      } else {
        active = false;
      }
    };
    if (active) {
      queue.push(exec);
    } else {
      active = true;
      exec();
    }
    return deferred;
  };
}
