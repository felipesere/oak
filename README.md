![example workflow](https://github.com/felipesere/oak/actions/workflows/actions.yml/badge.svg)

# oak
A fun Pokédex that combines timeless brands Pokémon and StarWars with a classical Shakespearean twist.
It's a server with a simple API that let's you grab information about Pokémon.
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

Tests interacting with either the server itself or any of the used APIs will need to be in `async` functions.
This is done by annotating the tests with `#[tokio::test]`, which is differnet from the standard Rust `#[test]` annotation.

There is also a bit of machinery around [mocks](src/mocks.rs) and [fixture data](fixtures).
The `mocks` module gives you a high-level interface to setup [WireMock](https://github.com/LukeMathWalker/wiremock-rs) for the used APIs and a matching, configured client.
The fixture data was captured as reference for the mocks.
This keeps the data faithful, but runs the risk of it going stale should the real API be updated.

## Running the application

There are couple of ways to run the application, depending on your goals.
If you just want to see what it does, jump straight to [Locally](#locally).
If you intend to run it through Docker because you don't want to install rust on your machine, see [Docker](#docker).

### Locally

First things first, let's ensure the code compiles by running

```sh
cargo build
```

This will give us a `debug` build of the server.
While faster to compile, the resulting binary might be a bit more bigger

Then, we can run the server and point it to the local [configuration](poke.yml) to get the necessary
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
for Mac, Windows, and various Linux distros. Remember that a gist is not official documentation!

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

## Using the API

Once the API is up and running (either locally or in Docker) you can interact with it using an HTTP client.
In these examples, we'll be using [httpie](https://httpie.io/) because the way its used on the command line resembles what one would expect from the HTTP protocol.

First, we are going to connect to a random route to see how the server responds:

```sh
http localhost:8000/not/a/route

HTTP/1.1 404 Not Found
content-type: application/json

{
    "message": "Route '/not/a/route' was not found",
    "help": "There are only two valid routes: '/pokemon/<name>' and '/pokemon/translated/<name>'",
    "examples": {
        "diglett_translated": "/pokemon/translated/diglett",
        "mewtwo": "/pokemon/mewtwo"
    }
}
```

We see the expected 404 Not Found status code, but there is some JSON in the response!
The `message` field states that `/not/a/route` not valid and the `help` field tells us which routes the server supports.
Finally, the repsonse shows two example routes under the `examples` field, one for Mewtwo and one for Diglett.
Let's run them:

```sh
http localhost:8000/pokemon/mewtwo

HTTP/1.1 200 OK
content-type: application/json

{
    "description": "It was created by a scientist after years of horrific gene splicing and DNA engineering experiments.",
    "habitat": "rare",
    "isLegendary": true,
    "name": "mewtwo"
}
```

and

```sh
http localhost:8000/pokemon/translated/diglett

HTTP/1.1 200 OK
content-type: application/json

{
    "description": "On plant roots,  lives about one yard underground where it feeds.Above ground,  it sometimes appears.",
    "habitat": "cave",
    "isLegendary": false,
    "name": "diglett"
}
```

> Notice: The FunTranslation API has a very narrow usage quota of 5 requests per hour! See [Caching of the PokeAPI and FunTranslation API](#caching-of-the-pokeapi-and-funtranslations-apiA)

That shows the two interesting endpoints on the API.

If you are keen try more examples, the `/pokemon/translated/<name>` endpoint reacts slightly differently for cave or legendary Pokemon. Instead of guessing which Pokemon fall into that category (_I guessed wrong a couple of times! `Geodude` lives in mountains, not caves!_) you can use the two scripts in `bin/`:

* `bin/cave-pokemon.sh` will print all Pokemon that inhabit caves according to PokeAPI
* `bin/legendaries.sh` will print all Pokemon that PokeAPI considers legendary

You will need to have `httpie` and [jq](https://stedolan.github.io/jq/) installed to run these.


## What I'd do differently for a production API

This backend was built in a few days with no outside influcence other than what I could gather from books or the internet.
While it reflects my past experience and current interests, there are certainly areas that I'd address differently
in a real-life production app.

### Caching of the PokeApi and FunTranslations API
The PokeAPI delivers pretty static data. Thankfully, Pokemons only change when new generations of the games are released.
As such any two users requesting details about the same Pokemon could be handed the same response.
There are possible solutions to address this at various layers:
* The PokeAPI client could have an internal cache that it builds up over time.
* The server module could alternatively also hold the cache, to keep the client free of any complex interdependencies
* The entire API can be put behind an HTTP-cache such as [Varnish](https://github.com/varnishcache/varnish-cache), [Squid](http://www.squid-cache.org/), or [Nginx](https://docs.nginx.com/nginx/admin-guide/content-cache/content-caching/).

The choice of what to cache depends on where use cases are coming from and who ends up being operationally responsible for the API.

The FunTranslations API is probably in most need of caching, as its free API has a very limited quota of 5 requests/hour.
That is so low that even with a single user we are very likely to hit the limit.
On the flip side, we have a very robust fallback for when the quota of translations is hit: we simply don't translate.
This makes the caching less cricial. That could change if we get negative user feedback due to untranslated requests!

I decided against pursuing any of these caching options to keep the code concise and correct.
Without knowing how successful our API is, its difficult to justify any complex caching strategies.
If necessary, I would advise for a simple cache either in the `server.rs` module or in any of the client modules.

### Metrics, logs, and more
As it stands, the logs are barely textual and there are no metrics or events at all.
That is OK for demonstrating that the API works in a bounded setting (e.g. developer laptops) or a small MVP environment.
Once the service goes live, the expectations of users increases dramatically and we'll need to monitor more aspects of our application.
I'd invest time in setting up the necessary code to gain insights such as:

* Monitor how frequently our endpoints are hit and what is the distribution of parameters (Pokemon). This can inform the above caching story!
* How fast is our API responding? Which parts of the stack dominate? Do we need to reach out to the PokeAPI to deal with capacity?
* We should monitor what errors occur accross the stack (i.e. Rusts `Result<T,E>` type) to see which parts are prone to errors and can use fallback strategies.

As it stands, operators have to look at our text-based log stream and potentially create their own extraction and ingestion into whatever tool they use.
We could aid this by producing our logs in a stable, predictable format such as JSON with annotated extra data.
There should be no need to setup intricate regex patterns to extract some information from our messages.
That kind of additional information should be added by the developers directly in the code.

This space is still in flux in the Rust ecosystem, though there seems to be a convergence on Tokios `tracing` and `tracing-subscriber` libraries.

### Configuration
Once observability is in place, operators can detect when there are issues, but as it stands there is little they can do.
Currently the configuration is partially baked into the application Docker image itself (`poke.yml`) or controlled by non-obvious, framework-dependent
environment variables such as `ROCKET_PORT=8000`.

In order to setup the configuration, I'd work closely with infrastructure with team to understand how they run other applications and what common patterns they follow:

* Do they build on configuration files per environment or is a template provided by the developers?
* What format do they use?
* Do they build a pattern well-known environemnt variables?
* Is there a service that handles configuration at runtime?

It's hard to tell from the outside what the correct answers are, but I'm sure with a couple video calls we'd be able to fit the `oak` server right in.

### API Versioning

At the moment there is only a single version of the API: latest. This is perfectly acceptable for a proof-of-concept. Before the API goes live though, I'd apply a version scheme. This allows the API to evolve over time to cover new use-cases, deprecated underused features, and react to security issues.

 There are various options we could consider:

* Versioned hosts, such as `v1.oak.io`
* Versioned paths, such as `oak.io/api/v1/...`
* Versioned content types for resources, such as `Content-Type: application/vnd+pokemon-v1+json`

### Security Considerations

Depending on how critical this API is to our business, I'd consider adding restricting access to authenticated applications and users.
This is definitely unnecessary for a proof-of-concept, but very important for live applications, particularly ones that have high traffic volumes and service-level agreements for clients.
Even though there is no sensitive data, being able to prevent bad actors from influencing customers -even indirectly, see [noisy neighbours](https://en.wikipedia.org/wiki/Cloud_computing_issues#Performance_interference_and_noisy_neighbors)- can help in maintain our reputation.
