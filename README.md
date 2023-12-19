# rust-snake

The snake game written in Rust.

- 動き回っているとお腹が空きます
- 満腹度が0になるとゲームオーバー
- 食べ物を食べるとお腹がふくれます
- 青＜黄色＜赤の順で栄養価が高く、満腹度が大きく上がります
- 白を食べると縮みます
- 3個食べると茶色の物体を生み出します
- 茶色の物体にぶつかるとゲームオーバー
- 自分自身をかじってしまってもゲームオーバー

## Requirement

- SDL 2
- Rust

## Key binginds

```
Keys:
    Up    : Move player up
    Down  : Move player down
    Left  : Move player left
    Right : Move player right
    Space : Restart when game over
```
