const events = require("./libnapi_events.so.node");

async function main() {
  // Takes the minimum log level, allowed values:
  // off, error, warn, info, debug, trace
  //
  // Throws if passing any other value.
  let test = new events.EventsTest("trace");

  // Produces two events, first with level `TRACE` and second with `INFO`;
  await test.produce_events();

  // Consumes the oldest event from the channel.
  let first_event = JSON.parse(await test.receive_event());
  console.log(first_event);
  // If the filter is set to a higher level, the channel only has one event and
  // this blocks forever.
  let second_event = JSON.parse(await test.receive_event());
  console.log(second_event);
}

main();
