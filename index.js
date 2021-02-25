const { loadBinding } = require('@node-rs/helper')

const events = loadBinding(__dirname, 'index', 'napi-events');

async function main() {
  // Takes the minimum log level, allowed values:
  // off, error, warn, info, debug, trace
  //
  // Throws if passing any other value.
  let test = new events.EventsTest("trace", (err, log) => {
    console.log(JSON.parse(log));
  });

  // Produces two events, first with level `TRACE` and second with `INFO`;
  await test.produce_events();

}

main().catch(console.error);
