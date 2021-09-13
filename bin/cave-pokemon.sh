#!/usr/bin/env zsh

# habitat 1 happens to be cave :) 
http https://pokeapi.co/api/v2/pokemon-habitat/1 | jq -r '.pokemon_species[] | .name' | sort
