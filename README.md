![example workflow](https://github.com/felipesere/oak/actions/workflows/actions.yml/badge.svg)

# oak
A fun Pokédex that combines timeless brands PokéMon and StarWars with a classical Shakespearean twist.


# TODO and thoughts
The list below is no strict order. Its a place to keep track of elements/structures I am considering
as they come up.

* [x] Build a client for the Pokemon API
  * [x] Serialize Pokemon JSON to a "PokemonAPI struct"
  * [x] Create a struct that represents `PokemonSettings` that produces a `PokemonClient`
  * [x] use reqwest to fire off any requests
    * [x] Use Wiremock for local testing

* [x] Find the right the flavour text in the right language and construct a smaller Pokemon type
  * [?] ...or do that with some custom Serde magic

* [ ] Build a client for the translations API
  * [x] Serialize a response from the Translations API
  * [x] use reqwest to fire off any requests
  * [x] Definitly cover the error of rate limiting!
     * [ ] Consider logging the remaining Rate-Limiting response header!
  * [x] Handle bad JSON
  * [ ] Add some rudimentary logging

* [ ] Server
  * [ ] Break server out into its own module
  * [x] Sketch out a basic server with a single endpoint that retrieves the Pokemon from the live PokeApi
      * [x] Write an endpoint `GET /pokemon/mewtwo` that responds with a hardcoded, boring string
      * [x] Write a type that serializes to the desiered JSON format
      * [x] Use new type in hardcoded response
      * [x] Untranslated response is good to go
   * [x] Sketch out translation endpoint
      * [x] Write an endpoint `GET /pokemon/translated/mewtwo` with a hardcoded response
      * [x] Wire in the Translation API client
      * [~] Apply rule for Yoda translation
          * [x] cave
          * [x] legendary
      * [x] Use shakespear translation

* [ ] See if I can lift `mocks::` into its own module/crate that can be used from all tests?

  * [ ] Server settings
    * [x] Sketch out a type
    * [ ] Read up on figmet to support multiple types of configurations, but at least ENV vars (e.g. port!) and YAML
    * [ ] Server settings for port
  * [ ] Consider elevating the tests to integration tests rather than unit tests

* [ ] Logging & Tracing
    * [x] Add some basic logs to the PokeApi
    * [ ] Consider using tokios `tracing` but it will need pretty manual setup
    * [ ] Consider outputing JSON rather than just a log file

* [x] Setup CI with Github Actions (is this still free?) or CircleCI
* [ ] review types and check which ones are `pub`/`pub(crate)` and document accordingly


## Thoughts

* Use wiremock to mock the two APIs
* Use Rocket 0.5 for the server itself
* Consider Clap to start and run the whole thing
* Decouple from web framework where possible without sacrificing legibility
* Serde for JSON
* Reqwest to talk to the backend APIs
* Consider constructing pretty errors with Miette
