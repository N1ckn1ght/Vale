## Vale

Vale is an open source engine that plays Ultimate Tic-Tac-Toe (UTTT) using minimax alrogithm and hand-crafted static evaluation.  
It's made in Rust, so it's blazing fast, of course.

![Showcase](https://github.com/N1ckn1ght/my-gif-collection-for-github/blob/master/vale_v0.1.1-beta(scaled).gif?raw=true)

Currently provides a console interface for user to interact with a board.  
Meant to be used via interface program using similar to UCI protocol, but comms are coming soon(tm).

## Progress

You can go and play it already! Check out the releases B-)  

Board - Complete  
Testing interface - Complete  
Eval - Hand-crafted, not tuned yet  
Engine - Negamax, pre-sort with TPV and heuristics, has LMR  
Comms - In development  
Docs - Sorry

## Sample games

Vale v0.1.1-beta (X, time 1000) - [Xing Ultimate Tic-Tac-Toe v3 ai Hard (O)](https://michaelxing.com/UltimateTTT/v3/ai/):  
```
Game ended | Victory: X
1. e4 e1 2. f3 i9 3. g8 b6 4. d8 c6 5. i8 g4 6. b1 e3 7. e7 e2 8. f4 h2 9. d4 b2 10. c1 h3 11. e9 d9 12. b9 f7 13. g3 a9 14. c7 g2 15. a6 a8 16. a5 c4 17. i2 g6 18. c8 g5 19. a4 c3 20. h9 f9 21. g7 a1 22. c9 h7 23. i3 g9 24. i1#
```

[Xing Ultimate Tic-Tac-Toe v3 ai Hard (X)](https://michaelxing.com/UltimateTTT/v3/ai/) - Vale v0.1.1-beta (O, time 1000):  
```
Draw guaranteed
1. e5 d4 2. a2 b6 3. f8 h6 4. d8 a6 5. a9 c7 6. h3 e9 7. e7 f2 8. i5 i6 9. h9 f9 10. i9 g9 11. a8 c6 12. i7 i1 13. h1 f1 14. h2 d6 15. a7 a1 16. c2 g4 17. b3 d7 18. b1 d1 19. a3 g7 20. c3 i8 21. h5 f5 22. g5 ...
```

Vale (X, time 1000) - [uttt.ai](https://github.com/arnowaczynski/utttai) (O, 1000 simulations, Argmax):  
```
Game ended | Victory: O
1. e4 d2 2. b4 f2 3. h4 e2 4. e6 e9 5. d9 a9 6. b9 f7 7. g3 a7 8. c2 g4 9. a2 c4 10. g2 b6 11. f8 h6 12. d8 a5 13. c5 g5 14. a4 b2 15. d4 c1 16. h3 d7 17. c3 g8 18. c6 g7 19. a1 a3 20. a8 a6 21. c8 i5 22. i6 i8 23. h5 f5 24. i4 i3 25. h9 f9 26. h8 d5 27. b5 e5 28. g9 c9 29. i7 i1 30. i2 g6 31. c7 g1 32. e8 h1#
```

Vale (X, time 10000) - [uttt.ai](https://github.com/arnowaczynski/utttai) (O, 1000 simulations, Argmax):  
```
Victory: O guaranteed
1. e4 d2 2. a5 c5 3. h4 f2 4. i4 g3 5. b7 e2 6. e6 e9 7. f9 h9 8. e8 e5 9. f6 i8 10. i6 i9 11. g9 c8 12. g6 c7 13. i2 h6 14. f8 g5 15. a4 b3 16. f7 g2 17. c6 g8 18. b5 f5 19. g4 ...
```

Vale (X, depth 2 (fullmoves)) - Vale (O, depth 2 (fullmoves)):
```
Game ended | Draw
1. e5 d4 2. c3 i8 3. i4 i1 4. h1 e3 5. e7 d3 6. a8 a4 7. a2 c5 8. g5 a5 9. c6 h7 10. d2 a6 11. a7 a3 12. b9 d9 13. c9 g7 14. c1 h3 15. f7 g3 16. c7 i2 17. h5 d6 18. c8 h6 19. d7 a1 20. c2 i5 21. g6 h2 22. e6 i9 23. i7 h8 24. e4 f3 25. a1
```

## License
Vale is licensed under the AGPL-3.0-or-later. See the LICENSE file for the full text.

## Thanks
Kivicat  
Konstantin_Russia  
[L140-beep](https://github.com/L140-beep)  
Valentin Lebedev  
...and many others
