![example workflow](https://github.com/felipesere/oak/actions/workflows/actions.yml/badge.svg)

# oak
A fun Pokédex that combines timeless brands Pokémon and StarWars with a classical Shakespearean twist.
Its a server with a simple API that you let's you grab information about Pokémon.
And if you are in a particularly fun mood, you can try the _translated_ endpoint which will
either use yoda or shakespearean English for the description.

## Setup

The server is written in Rust, so make sure you have that installed.
You can get it from [rustup](https://rustup.rs) or the right mechanism for your
operating system or distribution.
For macOS you can use homebrew with `brew install rust`, while for most Linux distributions and Windows
the path through `rustup` is recommended.

The code was built against stable Rust `1.54.0`, so make sure you are close to that version number (or higher!)
with:
```sh
rustc --version
rustc 1.54.0 (a178d0322 2021-07-26)
```

## Running the tests

Running the tests is as easy as `cargo test`.
`cargo` is _the_ Rust package manager (think "npm" for JavaScript or "maven" for Java) and also the place where
most other tools in the Rust ecosystem will hook into.

When running the tests, don't be surprised if the output is pretty sparse.
This is good! Cargo will only print the full output from assertions, logs, etc when there is a failure.

As of writing, the output looks somewhat like this (the order is non-deterministic):

```sh
$ cargo test
    Finished test [unoptimized + debuginfo] target(s) in 0.32s
     Running unittests (target/debug/deps/oak-c58fa18c6db2495b)

running 24 tests
test pokeapi::tests::cleanup_any_line_and_form_feed_characters_from_flavour_text ... ok
test pokeapi::tests::fails_if_there_is_no_english_flavour_text ... ok
test pokeapi::tests::reads_configuration ... ok
test pokeapi::tests::deserializes_ditto ... ok
test server::test::serializes_pokemon_responses_to_the_adequate_json ... ok
test translation::tests::deserializes_a_successful_translation ... ok
test pokeapi::tests::error_when_pokemon_isnt_real ... ok
test pokeapi::tests::response_for_ditto_is_missing_some_values ... ok
test translation::tests::reports_an_error_when_rate_limit_has_been_hit ... ok
test translation::tests::translates_a_simple_sentence_to_yoda_speak ... ok
test translation::tests::reports_an_error_for_bad_json ... ok
test translation::tests::translates_a_weird_sentence_to_shakespeare_english ... ok
test server::test::lets_users_know_when_pokemon_were_not_found ... ok
test server::test::other_errors_result_in_a_500_error ... ok
test pokeapi::tests::retrieves_mewtwo_from_pokeapi ... ok
test server::test::requesting_non_existing_routes_gives_a_helpful_message_with_examples ... ok
test server::test::when_requesting_a_translated_pokemon_fails ... ok
test server::test::when_translated_pokemon_does_not_exist_a_404_is_returned ... ok
test server::test::non_legendary_or_cave_pokemon_are_translated_to_shakespearan_english ... ok
test server::test::requesting_mewtwo_makes_a_call_to_the_pokemon_api ... ok
test server::test::when_asking_for_a_legendary_pokemon_the_translation_is_in_yoda_speak ... ok
test server::test::when_the_translation_fails_it_falls_back_to_the_standard_description ... ok
test server::test::when_asking_for_a_cave_pokemon_the_translation_is_in_yoda_speak ... ok
test pokeapi::tests::error_when_retrieving_ditto_takes_too_long ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
```

When adding your own tests, be mindufl that unlike standard rust tests, most of the tests interacting
with either the server itself or any of the used APIs will need be an `async` function.
The way this is achieve is annotating the tests with `#[tokio::test]`.

There is also a bit of machinery around [mocks](src/mocks.rs) and [fixture data](fixtures).
The `mocks` module gives you high-level interface to setup [WireMock](https://github.com/LukeMathWalker/wiremock-rs) for the used APIs and gives you matching, configured client.
The fixture data was captured and stored as reference and to not hand-roll the responses from the Mocks.
This keeps them faithful, at the potential risk of running stale should the API be updated.

## Running the application

There are couple of ways to run the application, depending on your goals.
If you just want to see what it does, jump straight to `Locallay`.
If you intend to run it through Docker because you don't want to install rust on your machine, see `Docker`.
Finally, if you want to deploy this application into a `Kubernetes` cluster, see that section.

### Locally

First things first, let's ensure the code compiles by running

```sh
cargo build
```

This will give us a `debug` build of the server.
While faster to compile, the resuling binary might be a bit more bloaty.

Then, we can run the server and point it to the local `[configuration](poke.yml)` to get the necessary
details for the backend:

```sh
./target/debug/oak --config poke.yml
```

You can combine these two steps with the handy `run` command from cargo, but be mindful on how to pass the `--config` parameter:

```sh
cargo run -- --config poke.yml
```

You should be greeted by the [Rocket](https://rocket.rs) splash screen.

### Docker

Running the `oak` server from within Docker is fairly easy, if you have Docker installed.
Installing Docker varies per operating system. [This gist](https://gist.github.com/rstacruz/297fc799f094f55d062b982f7dac9e41) gives a good overview
for Mac, Windows, and various Linux distros. Do be mindful that a gist is not official documentation!

Once you have docker installed, all you need to do is build it from the root of the repository:
```sh
docker build -t oak:latest .
```

This will produce an image that you can then run:
```sh
docker run -p 8000:8000 -e ROCKET_LOG_LEVEL=normal -ti oak:latest
```

In the above command, we expose port `8000` which is the default `ROCKET_PORT` and we raise the default logging
level to `normal` to see more activity with `ROCKET_LOG_LEVEL`.

The configuration for the PokeAPI and FunTranslation are placed in `poke.yml` which is baked into the
the Docker image itself.
If you want to change properties like timeouts, you'll have to remember to rebuild the image.

### Kubernetes

## What I'd do differently for a production API

* Caching of the PokeApi and translations

* Metrics, logs, and spans

* Configuration

* Pair
