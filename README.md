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

* [ ] Find the right the flavour text in the right language and construct a smaller Pokemon type
  * [?] ...or do that with some custom Serde magic

* [ ] Build a client for the translations API
  * [ ] Serialize a response from the Translations API
  * [ ] use reqwest to fire off any requests

* [ ] Server
  * [ ] Sketch out a basic server with a single endpoint that retrieves the Pokemon from the live PokeApi
      * [x] Write an endpoint `GET /pokemon/mewtwo` that responds with a hardcoded, boring string
      * [ ] Write a type that serializes to the desiered JSON format and replace the string :arrow_up:
  * [ ] Server settings for port

* [ ] Setup CI with Github Actions (is this still free?) or CircleCI
* [ ] review types and check which ones are `pub`/`pub(crate)` and document accordingly


## Thoughts

* Use wiremock to mock the two APIs
* Use Rocket 0.5 for the server itself
* Consider Clap to start and run the whole thing
* Decouple from web framework where possible without sacrificing legibility
* Serde for JSON
* Reqwest to talk to the backend APIs
* Consider constructing pretty errors with Miette
