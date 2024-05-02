### 1st Build of [Fushiguro Toji](https://telegram.me/FushiguroXTojiBot).

## Deploy
fill the `.env` file.
```env
BOT_TOKEN=your_bot_token
```
Install rust and run the following commands:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/Qewertyy/Facsimile
cd Facsimile
cargo build --release
cargo run --release
```
