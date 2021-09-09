# oak
A fun Pokédex that combines timeless brands PokéMon and StarWars with a classical Shakespearean twist.


# TODO and thoughts

* [ ] Build a client for the Pokemon API
  * [x] Serialize Pokemon JSON to a "PokemonAPI struct"
  * [ ] turn that struct into a smaller internal variant that already selects the right flavor language
  * [?] ...or do that with some custom Serde magic
  * [ ] Create a struct that represents `PokemonSettings` that produces a `PokemonClient`
  * [~] use reqwest to fire off any requests
    * [x] Use Wiremock for local testing
* [ ] Build a client for the translations API
  * [ ] Serialize a response from the Translations API
  * [ ] use reqwest to fire off any requests


## Thoughts

* Use wiremock to mock the two APIs
* Use Rocket 0.5 for the server itself
* Consider Clap to start and run the whole thing
* Decouple from web framework where possible without sacrificing legibility
* Serde for JSON
* Reqwest to talk to the backend APIs
* Consider constructing pretty errors with Miette
