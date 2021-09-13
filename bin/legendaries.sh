#!/usr/bin/env zsh

echo -n '{"query": "query samplePokeAPIquery { legendaries: pokemon_v2_pokemonspecies(where: {is_legendary: {_eq: true}}) { name } }"}' | http POST https://beta.pokeapi.co/graphql/v1beta | jq -r '.data.legendaries[] | .name' | sort
