To determine the maximum number of plies in a game of "Azul", we need to consider the worst-case scenario in terms of how many individual tile-taking actions can occur in each round.

1. **Factory Displays**: 
   - In a 2-player game, there are 5 factory displays.
   - In a 3-player game, there are 7 factory displays.
   - In a 4-player game, there are 9 factory displays.
   
   In the worst case, each factory display is emptied one tile at a time. This means a maximum of 4 plies per factory display (since each display starts with 4 tiles).

2. **Center**: 
   - After the factory displays are emptied, players can take tiles from the center. 
   - In the worst-case scenario, the center is emptied one tile at a time.

3. **Rounds**: 
   - The game consists of 5 rounds.

Considering a 4-player game for maximum plies:

1. Each of the 9 factory displays contributes 4 plies: \(9 \times 4 = 36\).
2. The center can have a maximum of \(9 \times 3 = 27\) tiles (since 1 tile from each factory display goes to the center when the other 3 are taken). So, a maximum of 27 plies from the center.
3. The total for one round is \(36 + 27 = 63\) plies.
4. Over 5 rounds: \(63 \times 5 = 315\).

Thus, the maximum number of plies in a 4-player game of "Azul" is 315. Note that this is a theoretical maximum, assuming the least efficient tile-taking strategy by all players. In practice, the number of plies would usually be much lower.