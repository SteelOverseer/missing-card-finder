# missing-card-finder

## Instructions
- Compile to windows with ```cargo build --target x86_64-pc-windows-gnu --release``` command
    - If not installed already, the compile target needs to be added with ```rustup target add x86_64-pc-windows-gnu```
- ```configuration.yaml``` file needs to be in the same directory as the executable.
- Formats and Decks will be processed in the order they appear in the config file.

## Description

Take an input of decklists and a collection. Find the difference and report on the cards in decks that are not in the collection.

Sample Collection Input:
```text
Total Qty,Reg Qty,Foil Qty,Card,Set,Mana Cost,Card Type,Color,Rarity,Mvid,Single Price,Single Foil Price,Total Price,Price Source,Notes
1,0,1,The Ur-Dragon,Commander 2017,4WUBRG,Legendary Creature  - Dragon Avatar,Gold,Mythic Rare,433289,,,,000001BF57777360,
1,0,1,"Sivitri, Dragon Master",Dominaria United Commander,2UB,Legendary Planeswalker — Sivitri,Gold,Mythic Rare,1257475,7.90,8.50,8.50,000001BF57777360,
1,0,1,"Sivitri, Dragon Master",Dominaria United Commander,2UB,Legendary Planeswalker — Sivitri,Gold,Mythic Rare,1257495,7.90,8.50,8.50,000001BF57777360,
```

Sample Deck Input:
```text
///mvid:1261055 qty:1 name:Aether Spellbomb loc:Deck
1 Aether Spellbomb
///mvid:205328 qty:4 name:Cranial Plating loc:Deck
4 Cranial Plating
///mvid:383222 qty:4 name:Darksteel Citadel loc:Deck
4 Darksteel Citadel
///mvid:1259539 qty:3 name:Forging the Anchor loc:Deck
3 Forging the Anchor
///mvid:222856 qty:4 name:Frogmite loc:Deck
4 Frogmite
///mvid:1241586 qty:1 name:Gingerbrute loc:Deck
1 Gingerbrute
///mvid:1261475 qty:2 name:Island loc:Deck
2 Island
```

Sample Output:
```text
/////////////////////////////////// Modern ///////////////////////////////////
-------- Affinity ** FOIL ** --------
1 Gingerbrute
2 Forging the Anchor
2 Treasure Vault
3 Sojourner's Companion
1 Damping Sphere
1 Pithing Needle
2 Tormod's Crypt
2 Tanglepool Bridge
1 Welding Jar
4 Mistvault Bridge
1 Haywire Mite
2 Patchwork Automaton
1 Memnite
1 Nettlecyst
4 Thought Monitor
3 Urza's Saga
1 Shadowspear
2 Hurkyl's Recall
3 Metallic Rebuke
1 Relic of Progenitus
4 Ornithopter
4 Thoughtcast
-------- DeathsShadow-Grixis --------
2 Unholy Heat
3 Ledger Shredder
4 Dragon's Rage Channeler
2 Unlicensed Hearse
2 Flusterstorm
1 Tourach, Dread Cantor
4 Ragavan, Nimble Pilferer
1 Kroxa, Titan of Death's Hunger
3 Expressive Iteration
1 Dress Down
1 Alpine Moon
```
