## Field Processors

Fields are like different layers of existence. A field can either be spatial or custom. 

A spatial field assigns players to *Field Processors* based on their spatial coordinates, such that, players that are close together wind up on the same *Field Processor*. This is useful for game logic that requires interactions from players that are close together. 

A custom field may or may not have grouping logic. This is useful for fields that can interact with other entities that are faraway or by some other criteria.


An example design could be to write fields like this:

Spatial Fields:
  - location: governs player location and phyiscs
  - combat: fighting interaction: using abilities, spells. This can be used for mobs, players, npcs etc
  - local-chat: only players in the immediate vicinity will see the chat message

No grouping:
  - interactable: interactable entities in the world. These interact only with players and don't interact with each other so don't need any grouping.
  - notification: only interacts with the client.

Custom Grouping:
  - guild-chat: big guilds can have their own processor and every member will be grouped together. smaller guilds can be grouped together and share the same processor



Field Processors interact with each other through:
 - Commands -> commands are RPC calls and represent events. They are guaranteed to be delivered. The caller can specify the target entity-field or a query that evaluates to a list of entity-fields. 
 - Queries -> A processor can ask the director to notify it when the data changes. This is not guaranteed tobe delivered. 
 
TBD: Define query/cmd rules, what can be queried, by whom and in what context/where.